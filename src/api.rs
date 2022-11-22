use crate::deserializer::timestamp;
use crate::entity::*;
use anyhow::{anyhow, Context as _, Result};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use reqwest::{
    header::{HeaderMap, CONTENT_TYPE},
    Method, Url,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

const ENTRY_POINT: &str = "https://api.bitflyer.com";

pub struct Client {
    client: reqwest::Client,
    api_key: String,
    hasher: Option<Hmac<Sha256>>,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Client {{ ... }}")
    }
}

impl Client {
    pub fn new() -> Result<Self> {
        let hasher = if let Ok(secret) = std::env::var("API_SECRET") {
            Some(Hmac::<Sha256>::new_from_slice(secret.as_bytes())?)
        } else {
            None
        };
        Ok(Self {
            client: reqwest::Client::new(),
            api_key: std::env::var("API_KEY").ok().unwrap_or_default(),
            hasher,
        })
    }

    #[tracing::instrument]
    pub async fn send<T>(&self, request: T) -> Result<<T as ApiRequest>::Response>
    where
        T: ApiRequest + std::fmt::Debug,
        <T as ApiRequest>::Response: for<'a> Deserialize<'a>,
    {
        let url = request.url()?;
        let response = if T::IS_PRIVATE {
            let timestamp = Utc::now().timestamp();
            let body = request.body()?;
            let data = format!(
                "{}{}{}{}{}",
                timestamp,
                T::METHOD.as_str(),
                T::PATH,
                url.query().map(|x| format!("?{x}")).unwrap_or_default(),
                body.clone().unwrap_or_default()
            );
            let mut hasher = self.hasher.clone().context("hasher is none")?;
            hasher.update(data.as_bytes());
            let hash = hasher.finalize().into_bytes();
            let hash = hash
                .iter()
                .map(|n| format!("{:02x}", n))
                .collect::<String>();
            let mut headers = HeaderMap::new();
            headers.insert("ACCESS-KEY", self.api_key.parse()?);
            headers.insert("ACCESS-TIMESTAMP", timestamp.to_string().parse()?);
            headers.insert("ACCESS-SIGN", hash.parse()?);
            if let Some(body) = body {
                headers.insert(CONTENT_TYPE, "application/json".parse()?);
                self.client
                    .request(T::METHOD, url)
                    .headers(headers)
                    .body(body)
                    .send()
                    .await?
            } else {
                self.client
                    .request(T::METHOD, url)
                    .headers(headers)
                    .send()
                    .await?
            }
        } else {
            self.client.request(T::METHOD, url).send().await?
        };
        if response.status().is_success() {
            let body = response.text().await?;
            let v: <T as ApiRequest>::Response = T::deserialize_response_body(&body)?;
            Ok(v)
        } else {
            Err(anyhow::anyhow!(
                "request is failed: status -> {}\nrequest -> {:?}\nrequest.body -> {:?}\nresponse -> {:?}",
                response.status(),
                request,
                request.body(),
                response.text().await
            ))
        }
    }
}

pub trait ApiRequest {
    const PATH: &'static str;
    const IS_PRIVATE: bool = false;
    const METHOD: Method = Method::GET;
    type Response: for<'a> Deserialize<'a>;

    fn url(&self) -> Result<Url> {
        let params = self.url_params();
        let params = params.iter().filter_map(|x| x.as_ref()).collect::<Vec<_>>();
        if params.is_empty() {
            Ok(Url::parse(&format!("{ENTRY_POINT}{}", Self::PATH))?)
        } else {
            Ok(Url::parse_with_params(
                &format!("{ENTRY_POINT}{}", Self::PATH),
                params,
            )?)
        }
    }

    fn url_params(&self) -> Vec<Option<(String, String)>> {
        vec![]
    }

    fn body(&self) -> Result<Option<String>> {
        Ok(None)
    }

    fn deserialize_response_body(body: &str) -> Result<Self::Response> {
        Ok(serde_json::from_str(body)?)
    }
}

pub trait QueryValue {
    fn to_query_parameter(&self, key: &str) -> Option<(String, String)>;
}

impl<T: ToString + Clone> QueryValue for Option<T> {
    fn to_query_parameter(&self, key: &str) -> Option<(String, String)> {
        self.clone().map(|x| (key.to_string(), x.to_string()))
    }
}

pub async fn send_api<T>(request: T) -> Result<<T as ApiRequest>::Response>
where
    T: ApiRequest + std::fmt::Debug,
    <T as ApiRequest>::Response: for<'a> Deserialize<'a>,
{
    let result = reqwest::get(request.url()?).await?;
    if result.status().is_success() {
        let body = result.text().await?;
        let v: <T as ApiRequest>::Response = serde_json::from_str(&body)?;
        Ok(v)
    } else {
        Err(anyhow::anyhow!(
            "request is failed: status -> {}\nurl -> {}",
            result.status(),
            request.url()?
        ))
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub struct Empty;

#[derive(Clone, Copy, Debug, Default)]
pub struct GetMarkets;
impl ApiRequest for GetMarkets {
    const PATH: &'static str = "/v1/markets";
    type Response = Vec<Market>;
}

#[derive(Clone, Debug, Default)]
pub struct GetBoard {
    pub product_code: Option<ProductCode>,
}
impl ApiRequest for GetBoard {
    const PATH: &'static str = "/v1/board";
    type Response = Board;

    fn url_params(&self) -> Vec<Option<(String, String)>> {
        vec![self.product_code.to_query_parameter("product_code")]
    }
}

#[derive(Clone, Debug, Default)]
pub struct GetTicker {
    pub product_code: Option<ProductCode>,
}
impl ApiRequest for GetTicker {
    const PATH: &'static str = "/v1/ticker";
    type Response = Ticker;

    fn url_params(&self) -> Vec<Option<(String, String)>> {
        vec![self.product_code.to_query_parameter("product_code")]
    }
}

#[derive(Clone, Debug, Default)]
pub struct GetExecutions {
    pub product_code: Option<ProductCode>,
    pub count: Option<u64>,
    pub before: Option<u64>,
    pub after: Option<u64>,
}
impl ApiRequest for GetExecutions {
    const PATH: &'static str = "/v1/executions";
    type Response = Vec<Execution>;

    fn url_params(&self) -> Vec<Option<(String, String)>> {
        vec![
            (self.product_code.to_query_parameter("product_code")),
            (self.count.to_query_parameter("count")),
            (self.before.to_query_parameter("before")),
            (self.after.to_query_parameter("after")),
        ]
    }
}

#[derive(Clone, Debug, Default)]
pub struct GetBoardState {
    pub product_code: Option<ProductCode>,
}
impl ApiRequest for GetBoardState {
    const PATH: &'static str = "/v1/getboardstate";
    type Response = BoardState;

    fn url_params(&self) -> Vec<Option<(String, String)>> {
        vec![(self.product_code.to_query_parameter("product_code"))]
    }
}

#[derive(Clone, Debug, Default)]
pub struct GetBoardHealth {
    pub product_code: Option<ProductCode>,
}
impl ApiRequest for GetBoardHealth {
    const PATH: &'static str = "/v1/gethealth";
    type Response = BoardHealth;

    fn url_params(&self) -> Vec<Option<(String, String)>> {
        vec![(self.product_code.to_query_parameter("product_code"))]
    }
}

#[derive(Clone, Debug, Default)]
pub struct GetPermissions;
impl ApiRequest for GetPermissions {
    const PATH: &'static str = "/v1/me/getpermissions";
    type Response = Vec<String>;
    const IS_PRIVATE: bool = true;
}

#[derive(Clone, Debug, Default)]
pub struct GetBalance;
impl ApiRequest for GetBalance {
    const PATH: &'static str = "/v1/me/getbalance";
    type Response = Vec<Balance>;
    const IS_PRIVATE: bool = true;
}

#[derive(Clone, Debug, Default)]
pub struct GetCollateral;
impl ApiRequest for GetCollateral {
    const PATH: &'static str = "/v1/me/getcollateral";
    type Response = Collateral;
    const IS_PRIVATE: bool = true;
}

#[derive(Clone, Debug, Default)]
pub struct GetCollateralAccounts;
impl ApiRequest for GetCollateralAccounts {
    const PATH: &'static str = "/v1/me/getcollateralaccounts";
    type Response = Vec<CollateralAccount>;
    const IS_PRIVATE: bool = true;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendChildOrderResponse {
    pub child_order_acceptance_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SendChildOrder {
    #[serde(flatten)]
    pub child_order_type: ChildOrderType,
    pub product_code: ProductCode,
    pub side: Side,
    pub size: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minute_to_expire: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
}
impl ApiRequest for SendChildOrder {
    const PATH: &'static str = "/v1/me/sendchildorder";
    const METHOD: Method = Method::POST;
    type Response = SendChildOrderResponse;
    const IS_PRIVATE: bool = true;

    fn body(&self) -> Result<Option<String>> {
        let json = serde_json::to_string(&self)?;
        Ok(Some(json))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CancelChildOrder {
    pub product_code: ProductCode,
    pub child_order_acceptance_id: String,
}
impl ApiRequest for CancelChildOrder {
    const PATH: &'static str = "/v1/me/cancelchildorder";
    const METHOD: Method = Method::POST;
    type Response = Empty;
    const IS_PRIVATE: bool = true;

    fn body(&self) -> Result<Option<String>> {
        let json = serde_json::to_string(&self)?;
        Ok(Some(json))
    }

    fn deserialize_response_body(body: &str) -> Result<Self::Response> {
        if body.is_empty() {
            Ok(Empty {})
        } else {
            Err(anyhow!("body is not empty"))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendParentOrderResponse {
    pub parent_order_acceptance_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SendParentOrder {
    #[serde(flatten)]
    pub order_method: ParentOrderMethod,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minute_to_expire: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
}
impl ApiRequest for SendParentOrder {
    const PATH: &'static str = "/v1/me/sendparentorder";
    const METHOD: Method = Method::POST;
    type Response = SendParentOrderResponse;
    const IS_PRIVATE: bool = true;

    fn body(&self) -> Result<Option<String>> {
        let json = serde_json::to_string(&self)?;
        Ok(Some(json))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CancelParentOrder {
    pub product_code: ProductCode,
    pub parent_order_acceptance_id: String,
}
impl ApiRequest for CancelParentOrder {
    const PATH: &'static str = "/v1/me/cancelparentorder";
    const METHOD: Method = Method::POST;
    type Response = Empty;
    const IS_PRIVATE: bool = true;

    fn body(&self) -> Result<Option<String>> {
        let json = serde_json::to_string(&self)?;
        Ok(Some(json))
    }

    fn deserialize_response_body(body: &str) -> Result<Self::Response> {
        if body.is_empty() {
            Ok(Empty {})
        } else {
            Err(anyhow!("body is not empty"))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CancelAllChildOrders {
    pub product_code: ProductCode,
}
impl ApiRequest for CancelAllChildOrders {
    const PATH: &'static str = "/v1/me/cancelallchildorders";
    const METHOD: Method = Method::POST;
    type Response = Empty;
    const IS_PRIVATE: bool = true;

    fn body(&self) -> Result<Option<String>> {
        let json = serde_json::to_string(&self)?;
        Ok(Some(json))
    }

    fn deserialize_response_body(body: &str) -> Result<Self::Response> {
        if body.is_empty() {
            Ok(Empty {})
        } else {
            Err(anyhow!("body is not empty"))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize)]
pub struct GetChildOrders {
    pub product_code: Option<ProductCode>,
    pub count: Option<u64>,
    pub before: Option<u64>,
    pub after: Option<u64>,
    pub child_order_acceptance_id: Option<String>,
    pub parent_order_id: Option<String>,
}
impl ApiRequest for GetChildOrders {
    const PATH: &'static str = "/v1/me/getchildorders";
    const METHOD: Method = Method::GET;
    type Response = Vec<ChildOrder>;
    const IS_PRIVATE: bool = true;

    fn url_params(&self) -> Vec<Option<(String, String)>> {
        vec![
            self.product_code.to_query_parameter("product_code"),
            self.count.to_query_parameter("count"),
            self.before.to_query_parameter("before"),
            self.after.to_query_parameter("after"),
            self.child_order_acceptance_id
                .to_query_parameter("child_order_acceptance_id"),
            self.parent_order_id.to_query_parameter("child_order_id"),
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct GetParentOrdersResponseParameter {
    pub id: u64,
    pub parent_order_id: String,
    pub product_code: ProductCode,
    pub side: ParentOrderSide,
    pub parent_order_type: ParentOrderType,
    pub price: Decimal,
    pub average_price: Decimal,
    pub size: Decimal,
    pub parent_order_state: OrderState,
    #[serde(with = "timestamp")]
    pub expire_date: DateTime<Utc>,
    #[serde(with = "timestamp")]
    pub parent_order_date: DateTime<Utc>,
    pub parent_order_acceptance_id: String,
    pub outstanding_size: Decimal,
    pub cancel_size: Decimal,
    pub executed_size: Decimal,
    pub total_commission: Decimal,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize)]
pub struct GetParentOrders {
    pub product_code: Option<ProductCode>,
    pub count: Option<u64>,
    pub before: Option<u64>,
    pub after: Option<u64>,
    pub parent_order_state: Option<OrderState>,
}
impl ApiRequest for GetParentOrders {
    const PATH: &'static str = "/v1/me/getparentorders";
    const METHOD: Method = Method::GET;
    type Response = Vec<GetParentOrdersResponseParameter>;
    const IS_PRIVATE: bool = true;

    fn url_params(&self) -> Vec<Option<(String, String)>> {
        vec![
            self.product_code.to_query_parameter("product_code"),
            self.count.to_query_parameter("count"),
            self.before.to_query_parameter("before"),
            self.after.to_query_parameter("after"),
            self.parent_order_state
                .to_query_parameter("parent_order_state"),
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct GetParentOrdersResponse {
    pub id: u64,
    pub parent_order_id: String,
    #[serde(with = "timestamp")]
    pub expire_date: DateTime<Utc>,
    pub time_in_force: TimeInForce,
    #[serde(flatten)]
    pub order_method: ParentOrderMethod,
    pub parent_order_acceptance_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize)]
pub struct GetParentOrder {
    pub parent_order_id: Option<String>,
    pub parent_order_acceptance_id: Option<String>,
}
impl ApiRequest for GetParentOrder {
    const PATH: &'static str = "/v1/me/getparentorder";
    const METHOD: Method = Method::GET;
    type Response = GetParentOrdersResponse;
    const IS_PRIVATE: bool = true;

    fn url_params(&self) -> Vec<Option<(String, String)>> {
        vec![
            self.parent_order_id.to_query_parameter("parent_order_id"),
            self.parent_order_acceptance_id
                .to_query_parameter("parent_order_acceptance_id"),
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize)]
pub struct GetPositions {}
impl ApiRequest for GetPositions {
    const PATH: &'static str = "/v1/me/getpositions";
    const METHOD: Method = Method::GET;
    type Response = Vec<Position>;
    const IS_PRIVATE: bool = true;

    fn url_params(&self) -> Vec<Option<(String, String)>> {
        vec![Some(ProductCode::FxBtcJpy).to_query_parameter("product_code")]
    }
}
