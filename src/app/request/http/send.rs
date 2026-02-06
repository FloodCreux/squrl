use crate::models::protocol::http::body::find_file_format_in_content_type;
use crate::models::request::Request;
use crate::models::response::RequestResponseError::CouldNotDecodeResponse;
use crate::models::response::{
    ImageResponse, RequestResponse, RequestResponseError, ResponseContent,
};
use parking_lot::RwLock;
use rayon::prelude::*;
use reqwest::header::CONTENT_TYPE;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{error, info, trace};

pub async fn send_http_request(
    request_builder: reqwest_middleware::RequestBuilder,
    local_request: Arc<RwLock<Request>>,
) -> Result<RequestResponse, RequestResponseError> {
    info!("Sending request");

    local_request.write().is_pending = true;

    let request = local_request.read();

    let cancellation_token = request.cancellation_token.clone();
    let timeout = tokio::time::sleep(Duration::from_millis(
        request.settings.timeout.as_u32() as u64
    ));

    let request_start = Instant::now();
    let elapsed_time: Duration;

    let mut response = tokio::select! {
        _ = cancellation_token.cancelled() => {
            elapsed_time = request_start.elapsed();
            build_sentinel_response("CANCELLED")
        },
        _ = timeout => {
            elapsed_time = request_start.elapsed();
            build_sentinel_response("TIMEOUT")
        },
        response = request_builder.send() => match response {
            Ok(resp) => {
                info!("Response received");

                elapsed_time = request_start.elapsed();

                let status_code = resp.status().to_string();

                let (headers, is_image) = extract_headers(resp.headers());
                let cookies = extract_cookies(&resp);

                let response_content = match is_image {
                    true => {
                        let content = resp.bytes().await.unwrap();
                        let image = image::load_from_memory(content.as_ref());

                        ResponseContent::Image(ImageResponse {
                            data: content.to_vec(),
                            image: image.ok(),
                        })
                    },
                    false => match resp.bytes().await {
                        Ok(bytes) => match String::from_utf8(bytes.to_vec()) {
                            Ok(mut result_body) => {
                                if let Some(file_format) = find_file_format_in_content_type(&headers) {
                                    if request.settings.pretty_print_response_content.as_bool() {
                                        match file_format.as_str() {
                                            "json" => {
                                                result_body = jsonxf::pretty_print(&result_body).unwrap_or(result_body);
                                            },
                                            _ => {},
                                        }
                                    }
                                }

                                ResponseContent::Body(result_body)
                            },
                            Err(_) => ResponseContent::Body(format!("{:#X?}", bytes))
                        },
                        Err(_) => return Err(CouldNotDecodeResponse)
                    },
                };

                RequestResponse {
                    duration: None,
                    status_code: Some(status_code),
                    content: Some(response_content),
                    cookies: Some(cookies),
                    headers,
                }
            },
            Err(err) => {
                error!("Sending error: {}", err);

                elapsed_time = request_start.elapsed();

                let response_status_code;

                if let Some(status_code) = err.status() {
                    response_status_code = Some(status_code.to_string());
                } else {
                    response_status_code = None;
                }

                let result_body = ResponseContent::Body(err.to_string());


                RequestResponse {
                    duration: None,
                    status_code: response_status_code,
                    content: Some(result_body),
                    cookies: None,
                    headers: vec![],
                }
            },
        },
    };

    response.duration = Some(format!("{:?}", elapsed_time));

    trace!("Request sent");

    Ok(response)
}

fn build_sentinel_response(status_code: &str) -> RequestResponse {
    RequestResponse {
        duration: None,
        status_code: Some(String::from(status_code)),
        content: None,
        cookies: None,
        headers: vec![],
    }
}

fn extract_headers(response_headers: &reqwest::header::HeaderMap) -> (Vec<(String, String)>, bool) {
    let mut is_image = false;
    let result = response_headers
        .iter()
        .map(|(header_name, header_value)| {
            let value = header_value.to_str().unwrap_or("").to_string();

            if header_name == CONTENT_TYPE && value.starts_with("image/") {
                is_image = true;
            }

            (header_name.to_string(), value)
        })
        .collect();

    (result, is_image)
}

fn extract_cookies(response: &reqwest::Response) -> String {
    response
        .cookies()
        .par_bridge()
        .map(|cookie| format!("{}: {}", cookie.name(), cookie.value()))
        .collect::<Vec<String>>()
        .join("\n")
}
