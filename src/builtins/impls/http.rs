// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Builtins used to make HTTP request

use std::{collections::HashMap, time::Duration};

use anyhow::{bail, Result};
use duration_str::deserialize_duration;
use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions, MokaManager};
use reqwest::{header::HeaderMap, redirect::Policy, Client, Method};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, RequestBuilder};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
enum Timeout {
    #[serde(deserialize_with = "deserialize_duration")]
    TimeString(Duration),
    Nanosec(u64),
}

///representation of a http request
#[derive(Deserialize, Serialize, Debug)]
pub struct Request {
    url: String,
    #[serde(with = "http_serde::method")]
    method: Method,
    body: Option<serde_json::Value>,
    raw_body: Option<String>,
    headers: Option<HashMap<String, String>>,
    enable_redirect: Option<bool>,
    force_json_decode: Option<bool>,
    force_yaml_decode: Option<bool>,
    tls_use_system_cert: Option<bool>,
    tls_ca_cert: Option<String>,
    tls_ca_cert_file: Option<String>,
    tls_ca_cert_env_variable: Option<String>,
    tls_client_key: Option<String>,
    tls_client_key_file: Option<String>,
    tls_client_key_env_variable: Option<String>,
    timeout: Option<Timeout>,
    tls_insecure_skip_verify: Option<bool>,
    tls_server_name: Option<String>,
    cache: Option<bool>,
    force_cache: Option<bool>,
    force_cache_duration_seconds: Option<bool>,
    caching_mode: Option<String>,
    raise_error: Option<bool>,
    max_retry_atempts: Option<u32>,
}

/// representation of the response body type
#[derive(Deserialize, Serialize, Debug)]
pub enum BodyType {
    ///json body
    Json(serde_json::Value),
    ///yaml body
    Yaml(serde_yaml::Value),
}

///representation of a http response
#[derive(Deserialize, Serialize, Debug)]
pub struct Response {
    status: String,
    status_code: u16,
    body: Option<BodyType>,
    raw_body: String,
    #[serde(with = "http_serde::header_map")]
    headers: HeaderMap,
    error: HashMap<String, u16>,
}

fn unimplemented_option(data: &Request) -> Result<()> {
    if let Some(_op) = data.raise_error {
        bail!("option unimplemented!")
    }
    if let Some(_op) = &data.tls_ca_cert {
        bail!("option unimplemented!")
    }
    if let Some(_op) = &data.tls_client_key {
        bail!("option unimplemented!")
    }
    if let Some(_op) = &data.tls_server_name {
        bail!("option unimplemented!")
    }
    if let Some(_op) = data.tls_use_system_cert {
        bail!("option unimplemented!")
    }
    if let Some(_op) = &data.tls_ca_cert_file {
        bail!("option unimplemented!")
    }
    if let Some(_op) = &data.tls_client_key_file {
        bail!("option unimplemented!")
    }
    if let Some(_op) = &data.tls_ca_cert_env_variable {
        bail!("option unimplemented!")
    }
    if let Some(_op) = &data.tls_client_key_env_variable {
        bail!("option unimplemented!")
    }
    if let Some(_op) = &data.tls_insecure_skip_verify {
        bail!("option unimplemented!")
    }
    if let Some(_op) = &data.caching_mode {
        bail!("option unimplemented!")
    }
    if let Some(_op) = &data.force_cache_duration_seconds {
        bail!("option unimplemented!")
    }
    Ok(())
}

fn decode_body(data: &Request, headers: &HeaderMap, raw_body: &str) -> Result<Option<BodyType>> {
    let body = if raw_body.is_empty() {
        None
    } else if let Some(header) = headers.get("Content-Type") {
        match header.to_str()? {
            "application/json" => Some(serde_json::from_str(raw_body)?),
            "application/yaml" | "application/x-yaml" => Some(serde_yaml::from_str(raw_body)?),
            _ => None,
        }
    } else if let Some(true) = data.force_json_decode {
        Some(serde_json::from_str(raw_body)?)
    } else if let Some(true) = data.force_yaml_decode {
        Some(serde_yaml::from_str(raw_body)?)
    } else {
        None
    };
    Ok(body)
}

fn build_client(data: &Request) -> Result<ClientWithMiddleware> {
    let mut client_builder = Client::builder();
    if let Some(false) = data.enable_redirect {
        client_builder = client_builder.redirect(Policy::none());
    }
    let client = client_builder.build()?;
    let mut client_builder = ClientBuilder::new(client);
    if let Some(retry) = data.max_retry_atempts {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(retry);
        client_builder =
            client_builder.with(RetryTransientMiddleware::new_with_policy(retry_policy));
    }
    if let Some(true) = data.cache {
        let mode = if let Some(true) = data.force_cache {
            CacheMode::ForceCache
        } else {
            CacheMode::Default
        };

        client_builder = client_builder.with(Cache(HttpCache {
            mode,
            manager: MokaManager::default(),
            options: HttpCacheOptions::default(),
        }));
    }
    Ok(client_builder.build())
}

fn build_request(data: &Request, client: ClientWithMiddleware) -> Result<RequestBuilder> {
    let mut request_builder = client.request(data.method.clone(), &data.url);
    if let Some(timeout) = &data.timeout {
        match timeout {
            Timeout::TimeString(n) => request_builder = request_builder.timeout(*n),
            Timeout::Nanosec(n) => {
                request_builder = request_builder.timeout(Duration::from_nanos(*n));
            }
        }
    }
    if let Some(headers) = &data.headers {
        request_builder = request_builder.headers(headers.try_into()?);
    }
    if let Some(body) = &data.body {
        request_builder = request_builder.json(&body);
    }
    if let Some(raw_body) = data.raw_body.clone() {
        request_builder = request_builder.body(raw_body);
    }
    Ok(request_builder)
}

/// Returns a HTTP response to the given HTTP request.
#[tracing::instrument(name = "http.send", err)]
pub async fn send(data: Request) -> Result<Response> {
    unimplemented_option(&data)?;
    let client = build_client(&data)?;

    let request = build_request(&data, client)?;
    let resp = request.send().await?;

    //extract data from response
    let mut status = resp.status().as_str().to_string();
    if let Some(reason) = resp.status().canonical_reason() {
        status = status + " " + reason;
    }
    let status_code = if let Some(false) = data.raise_error {
        0
    } else {
        resp.status().as_u16()
    };
    let error = HashMap::new();
    let headers = resp.headers().clone();
    let raw_body = resp.text().await?;
    let body = decode_body(&data, &headers, &raw_body)?;
    Ok(Response {
        status,
        status_code,
        body,
        raw_body,
        headers,
        error,
    })
}
