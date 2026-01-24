use anyhow::{Context, Result};
use axum::body::Bytes;
use multer::Multipart;

pub async fn parse_multipart_sms(
    boundary: &str,
    body: Bytes,
) -> Result<(serde_json::Value, Vec<u8>)> {
    let stream = futures_util::stream::once(async move { Ok::<_, std::io::Error>(body) });
    let mut multipart = Multipart::new(stream, boundary);

    let mut json_data: Option<serde_json::Value> = None;
    let mut sms_payload: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await? {
        let content_type = field.content_type().map(|m| m.to_string());

        match content_type.as_deref() {
            Some("application/json") => {
                let data = field.bytes().await?;
                json_data = Some(serde_json::from_slice(&data)?);
            }
            Some("application/vnd.3gpp.sms") => {
                let data = field.bytes().await?;
                sms_payload = Some(data.to_vec());
            }
            _ => {
                field.bytes().await?;
            }
        }
    }

    let json = json_data.context("Missing JSON part in multipart request")?;
    let payload = sms_payload.context("Missing SMS payload in multipart request")?;

    Ok((json, payload))
}
