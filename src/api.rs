use anyhow::Ok;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BaseResult {
    pub success: bool,
    pub message: String,
    pub code: String,
    #[serde(rename(serialize = "RequestId", deserialize = "RequestId"))]
    pub request_id: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenData {
    token: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenResult {
    #[serde(flatten)]
    base: BaseResult,

    // 返回错误可能没有data数据
    data: Option<TokenData>,
}

// https://doc.sm.ms/#api-_
pub async fn token(username: String, password: String) -> anyhow::Result<String> {
    let params = [("username", username), ("password", password)];

    let client = reqwest::Client::new();
    let res = client
        .post("https://sm.ms/api/v2/token")
        .form(&params)
        .send()
        .await?;

    let d = res.json::<TokenResult>().await?;

    // dbg!("{:?}", &d);

    if !d.base.success {
        anyhow::bail!(d.base.message);
    }

    Ok(d.data.unwrap().token)
}

#[derive(Debug, Deserialize)]
pub struct ProfileData {
    pub username: String,
    pub email: String,
    pub role: String,
    pub group_expire: String,
    pub email_verified: u8,
    pub disk_usage: String,
    pub disk_limit: String,
    pub disk_usage_raw: usize,
    pub disk_limit_raw: usize,
}

#[derive(Debug, Deserialize)]
pub struct ProfileResult {
    #[serde(flatten)]
    base: BaseResult,

    // 返回错误可能没有data数据
    data: Option<ProfileData>,
}

pub async fn profile(token: String) -> anyhow::Result<ProfileData> {
    let client = reqwest::Client::new();
    let res = client
        .post("https://sm.ms/api/v2/profile")
        .header("Authorization", token)
        .send()
        .await?;

    let d = res.json::<ProfileResult>().await?;

    // dbg!("{:?}", &d);

    if !d.base.success {
        anyhow::bail!(d.base.message);
    }

    Ok(d.data.unwrap())
}

pub async fn delete_image(token: String, hash: String) -> anyhow::Result<bool> {
    let client = reqwest::Client::new();
    let res = client
        .get(format!("https://sm.ms/api/v2/delete/{}", hash))
        .header("Authorization", token)
        .send()
        .await?;

    let d = res.json::<BaseResult>().await?;

    // dbg!("{:?}", &d);

    if !d.success {
        anyhow::bail!(d.message);
    }

    Ok(true)
}

#[derive(Debug, Deserialize)]
pub struct UploadHistoryData {
    pub width: i32,
    pub height: i32,
    pub filename: String,
    pub storename: String,
    pub size: usize,
    pub path: String,
    pub hash: String,
    pub created_at: String,
    pub url: String,
    pub delete: String,
    pub page: String,
}

#[derive(Debug, Deserialize)]
pub struct UploadHistoryResult {
    #[serde(flatten)]
    base: BaseResult,

    // 返回错误可能没有data数据
    data: Option<Vec<UploadHistoryData>>,
}

pub async fn upload_history(token: String, page: u32) -> anyhow::Result<Vec<UploadHistoryData>> {
    let client = reqwest::Client::new();
    let res = client
        .get(format!("https://sm.ms/api/v2/upload_history?page={}", page))
        .header("Authorization", token)
        .send()
        .await?;

    let mut d = res.json::<UploadHistoryResult>().await?;

    if !d.base.success {
        anyhow::bail!(d.base.message);
    }
    d.data.as_mut().unwrap().reverse();
    Ok(d.data.unwrap())
}
