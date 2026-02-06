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

                let response_content = decode_response_body(
                    resp,
                    &headers,
                    is_image,
                    request.settings.pretty_print_response_content.as_bool(),
                )
                    .await?;

                RequestResponse {
                    duration: None,
                    status_code: Some(status_code),
                    content: Some(response_content),
                    cookies: Some(cookies),
                    headers,
                }
            },
            Err(err) => {
                elapsed_time = request_start.elapsed();
                build_error_response(&err)
            }
        },
    };

    response.duration = Some(format!("{:?}", elapsed_time));

    trace!("Request sent");

    Ok(response)
}

fn build_error_response(err: &reqwest_middleware::Error) -> RequestResponse {
    error!("Sending error: {}", err);

    let response_status_code = err.status().map(|s| s.to_string());
    let result_body = ResponseContent::Body(err.to_string());

    RequestResponse {
        duration: None,
        status_code: response_status_code,
        content: Some(result_body),
        cookies: None,
        headers: vec![],
    }
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

async fn decode_response_body(
    response: reqwest::Response,
    headers: &Vec<(String, String)>,
    is_image: bool,
    pretty_print: bool,
) -> Result<ResponseContent, RequestResponseError> {
    let content = if is_image {
        let content = response.bytes().await.unwrap();
        let image = image::load_from_memory(content.as_ref());

        ResponseContent::Image(ImageResponse {
            data: content.to_vec(),
            image: image.ok(),
        })
    } else {
        match response.bytes().await {
            Ok(bytes) => match String::from_utf8(bytes.to_vec()) {
                Ok(mut result_body) => {
                    if pretty_print {
                        if let Some(file_format) = find_file_format_in_content_type(headers) {
                            if file_format == "json" {
                                result_body =
                                    jsonxf::pretty_print(&result_body).unwrap_or(result_body);
                            }
                        }
                    }

                    ResponseContent::Body(result_body)
                }
                Err(_) => ResponseContent::Body(format!("{:#X?}", bytes)),
            },
            Err(_) => return Err(CouldNotDecodeResponse),
        }
    };

    Ok(content)
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
