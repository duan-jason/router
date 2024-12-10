// JASON customization: get labels - consumerName, roles, correlationId, cid, azure_region
//
// Usage: 
//       (azure_region, consumer_name, role_id, correlation_id, cid) = uhg_custom::get_uhg_labels(context, headers);  // JASON customization
//
//      "consumerName" = consumer_name,
//      "roles" = role_id,
//      "correlationId" = correlation_id,
//      "cid" = cid,
//      "azure_region" = azure_region,
//
use http::HeaderMap;
use crate::Context;

pub fn get_uhg_azure_region() -> String {
    let azure_region = std::env::var("AZURE_REGION").unwrap_or(String::from(""));

    azure_region
}

fn get_uhg_value(headers:  Option<&HeaderMap>, context: Option<&Context>, header_name: &str, context_name: &str) -> String {
    let mut v = String::from("");
    
    if headers.is_some() {
        v = match headers.unwrap().get(header_name) {
            Some(s) => std::str::from_utf8(s.as_bytes()).unwrap_or_default().to_string(),
            None => String::from("")
        };
    };

    if v.len() == 0 && context.is_some() {
        v = match context.unwrap().get::<_, String>(context_name) {
                Ok(_v) => _v.unwrap_or_default(),
                _ => String::from("")
        }
    }

    v
}

pub fn get_uhg_labels(headers: Option<&HeaderMap>, context: Option<&Context>) -> (String, String, String, String, String) {
    let azure_region = get_uhg_azure_region();
    let consumer_name = get_uhg_value(headers, context, "consumername", "x-consumer-name");
    let role_id = get_uhg_value(headers, context, "roleid", "x-roles");
    let correlation_id = get_uhg_value(headers, context, "X-Correlation-Id", "x-correlation-id");
    let cid = get_uhg_value(headers, context, "Optum-Cid-Ext", "x-cid");

    (azure_region, consumer_name, role_id, correlation_id, cid)
}