use crate::app_settings::AppSettings;
use base64::{engine::general_purpose, Engine as _};
use serde::Deserialize;
use std::time::Duration;

const MAX_TEXT_INPUT_CHARS: usize = 12_000;
const MAX_IMAGE_INPUT_BYTES: usize = 8 * 1024 * 1024;
const MAX_IMAGE_OUTPUT_TOKENS: u16 = 4096;
const PROVIDER_OPENAI_COMPATIBLE: &str = "openai_compatible";
const PROVIDER_DEEPSEEK: &str = "deepseek";
const PROVIDER_XIAOMI_MIMO: &str = "xiaomi_mimo";
const PROVIDER_KIMI: &str = "kimi";
const PROVIDER_GLM: &str = "glm";
const PROVIDER_MINIMAX: &str = "minimax";
const PROVIDER_QWEN: &str = "qwen";

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Deserialize)]
struct ChatChoiceMessage {
    content: Option<String>,
}

pub async fn run_text_action(
    settings: &AppSettings,
    api_key: &str,
    action: &str,
    selected_text: &str,
    source_language: &str,
    target_language: &str,
) -> Result<String, String> {
    if selected_text.chars().count() > MAX_TEXT_INPUT_CHARS {
        return Err("选中文本过长，请缩短到 12000 字以内后再试".to_string());
    }

    let (system_prompt, user_prefix) = text_prompts(action, source_language, target_language)?;
    let endpoint = chat_completions_endpoint(settings, api_key, AiModelKind::Text)?;
    let model = effective_text_model(settings)?;
    let request =
        text_chat_request(settings, AiModelKind::Text, model, system_prompt, user_prefix, selected_text);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(settings.ai_timeout_seconds.into()))
        .build()
        .map_err(|_| "初始化 AI 请求客户端失败".to_string())?;

    let response = client
        .post(endpoint)
        .bearer_auth(api_key)
        .json(&request)
        .send()
        .await
        .map_err(map_request_error)?;

    let status = response.status();
    if !status.is_success() {
        return Err(map_http_status(status.as_u16()));
    }

    let body = response
        .json::<ChatCompletionResponse>()
        .await
        .map_err(|_| {
            "AI 返回格式无法解析，请检查 Base URL 和模型是否兼容 OpenAI chat/completions"
                .to_string()
        })?;

    body.choices
        .into_iter()
        .find_map(|choice| choice.message.content)
        .map(|content| content.trim().to_string())
        .filter(|content| !content.is_empty())
        .ok_or_else(|| "AI 返回为空，请稍后重试或检查模型配置".to_string())
}

pub async fn run_input_text_action(
    settings: &AppSettings,
    api_key: &str,
    action: &str,
    selected_text: &str,
    source_language: &str,
    target_language: &str,
) -> Result<String, String> {
    if selected_text.chars().count() > MAX_TEXT_INPUT_CHARS {
        return Err("输入文本过长，请缩短到 12000 字以内后再试".to_string());
    }

    let (system_prompt, user_prefix) = text_prompts(action, source_language, target_language)?;
    let endpoint = chat_completions_endpoint(settings, api_key, AiModelKind::Input)?;
    let model = effective_input_model(settings)?;
    let request = text_chat_request(
        settings,
        AiModelKind::Input,
        model,
        system_prompt,
        user_prefix,
        selected_text,
    );

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(settings.ai_timeout_seconds.into()))
        .build()
        .map_err(|_| "初始化 AI 请求客户端失败".to_string())?;

    let response = client
        .post(endpoint)
        .bearer_auth(api_key)
        .json(&request)
        .send()
        .await
        .map_err(map_request_error)?;

    let status = response.status();
    if !status.is_success() {
        return Err(map_http_status(status.as_u16()));
    }

    let body = response
        .json::<ChatCompletionResponse>()
        .await
        .map_err(|_| {
            "AI 返回格式无法解析，请检查输入模型 Base URL 和模型是否兼容 OpenAI chat/completions"
                .to_string()
        })?;

    body.choices
        .into_iter()
        .find_map(|choice| choice.message.content)
        .map(|content| content.trim().to_string())
        .filter(|content| !content.is_empty())
        .ok_or_else(|| "AI 返回为空，请稍后重试或检查输入模型配置".to_string())
}

pub async fn run_image_action(
    settings: &AppSettings,
    api_key: &str,
    action: &str,
    png_bytes: &[u8],
    source_language: &str,
    target_language: &str,
) -> Result<String, String> {
    if png_bytes.is_empty() {
        return Err("截图图片为空，请重新框选后再试".to_string());
    }
    if png_bytes.len() > MAX_IMAGE_INPUT_BYTES {
        return Err("截图图片过大，请缩小框选区域后再试".to_string());
    }

    let (system_prompt, user_prompt) = image_prompts(action, source_language, target_language)?;
    let endpoint = chat_completions_endpoint(settings, api_key, AiModelKind::Vision)?;
    let model = effective_vision_model(settings)?;
    let encoded_image = general_purpose::STANDARD.encode(png_bytes);
    let image_url = format!("data:image/png;base64,{encoded_image}");
    let request = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": system_prompt,
            },
            {
                "role": "user",
                "content": [
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": image_url,
                        },
                    },
                    {
                        "type": "text",
                        "text": user_prompt,
                    },
                ],
            },
        ],
        "temperature": 0.0,
        "max_tokens": MAX_IMAGE_OUTPUT_TOKENS,
    });

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(settings.ai_timeout_seconds.into()))
        .build()
        .map_err(|_| "初始化 AI 请求客户端失败".to_string())?;

    let response = client
        .post(endpoint)
        .bearer_auth(api_key)
        .json(&request)
        .send()
        .await
        .map_err(map_request_error)?;

    let status = response.status();
    if !status.is_success() {
        return Err(map_image_http_status(status.as_u16()));
    }

    let body = response
        .json::<ChatCompletionResponse>()
        .await
        .map_err(|_| {
            "AI 返回格式无法解析，请检查 Base URL 和视觉模型是否兼容 OpenAI chat/completions"
                .to_string()
        })?;

    body.choices
        .into_iter()
        .find_map(|choice| choice.message.content)
        .map(|content| content.trim().to_string())
        .filter(|content| !content.is_empty())
        .ok_or_else(|| "AI 返回为空，请稍后重试或检查视觉模型配置".to_string())
}

fn text_prompts(
    action: &str,
    source_language: &str,
    target_language: &str,
) -> Result<(String, String), String> {
    let source = translation_source_instruction(source_language);
    let target = translation_language_label(target_language);
    match action {
        "translate" => Ok((
            format!("你是 QuickPick 的划词翻译助手。{source}，并翻译成自然、准确的{target}。只输出译文，不添加解释。保留专有名词、代码、URL 和数字格式。"),
            format!("{source}，然后翻译为{target}。只输出译文："),
        )),
        "summarize" => Ok((
            "你是 QuickPick 的划词总结助手。用简体中文概括用户提供的文本，保留关键事实、术语和结论。只输出摘要。".to_string(),
            "请总结以下文本：".to_string(),
        )),
        _ => Err("未知的 AI 操作类型".to_string()),
    }
}

fn image_prompts(
    action: &str,
    source_language: &str,
    target_language: &str,
) -> Result<(String, String), String> {
    let source = translation_source_instruction(source_language);
    let target = translation_language_label(target_language);
    match action {
        "extract" => Ok((
            "你是 QuickPick 的截图文字提取助手。识别图片中所有清晰可见的文字，按自然阅读顺序输出原文。不要编造看不清的内容；没有可识别文字时说明未识别到文字。".to_string(),
            "请提取这张截图中的所有文字，只输出识别结果。".to_string(),
        )),
        "translate" => Ok((
            format!("你是 QuickPick 的截图文字翻译器。只做一件事：读取图片中所有清晰可辨的文字，{source}，然后直接翻译为自然、准确的{target}。禁止描述图片内容，禁止说明识别过程，禁止输出“图片中显示...”“我看到了...”等说明。保留专有名词、数字、URL 和代码格式。如果图片没有可识别文字，只输出“未识别到文字”。"),
            format!("直接翻译这张截图中的文字为{target}。只输出译文，不要输出原文对照，也不要描述图片内容或识别过程。"),
        )),
        _ => Err("未知的图片 AI 操作类型".to_string()),
    }
}

fn text_chat_request(
    settings: &AppSettings,
    kind: AiModelKind,
    model: String,
    system_prompt: String,
    user_prefix: String,
    selected_text: &str,
) -> serde_json::Value {
    let mut request = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": system_prompt,
            },
            {
                "role": "user",
                "content": format!("{user_prefix}\n\n{selected_text}"),
            },
        ],
        "temperature": 0.2,
    });

    let provider_id = match kind {
        AiModelKind::Text => text_provider_id(settings),
        AiModelKind::Input => input_provider_id(settings),
        AiModelKind::Vision => vision_provider_id(settings),
    };
    if provider_id == PROVIDER_DEEPSEEK {
        request["thinking"] = serde_json::json!({ "type": "disabled" });
    }

    request
}

#[derive(Clone, Copy)]
enum AiModelKind {
    Text,
    Vision,
    Input,
}

fn chat_completions_endpoint(
    settings: &AppSettings,
    api_key: &str,
    kind: AiModelKind,
) -> Result<String, String> {
    let base_url = match kind {
        AiModelKind::Text => effective_text_base_url(settings, api_key),
        AiModelKind::Vision => effective_vision_base_url(settings, api_key),
        AiModelKind::Input => effective_input_base_url(settings, api_key),
    };
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err("Base URL 未配置".to_string());
    }

    if !(trimmed.starts_with("https://") || trimmed.starts_with("http://")) {
        return Err("Base URL 必须以 http:// 或 https:// 开头".to_string());
    }

    Ok(format!("{trimmed}/chat/completions"))
}

fn effective_text_base_url(settings: &AppSettings, api_key: &str) -> String {
    let configured = settings.text_ai_base_url.trim();
    if !configured.is_empty() {
        return configured.to_string();
    }

    match text_provider_id(settings) {
        PROVIDER_DEEPSEEK => "https://api.deepseek.com".to_string(),
        PROVIDER_XIAOMI_MIMO if api_key.trim().starts_with("tp-") => {
            "https://token-plan-cn.xiaomimimo.com/v1".to_string()
        }
        PROVIDER_XIAOMI_MIMO => "https://api.xiaomimimo.com/v1".to_string(),
        PROVIDER_KIMI => "https://api.moonshot.cn/v1".to_string(),
        PROVIDER_GLM => "https://open.bigmodel.cn/api/paas/v4".to_string(),
        PROVIDER_MINIMAX => "https://api.minimaxi.com/v1".to_string(),
        PROVIDER_QWEN => "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
        _ => String::new(),
    }
}

fn effective_vision_base_url(settings: &AppSettings, api_key: &str) -> String {
    let configured = settings.vision_ai_base_url.trim();
    if !configured.is_empty() {
        return configured.to_string();
    }

    match vision_provider_id(settings) {
        PROVIDER_XIAOMI_MIMO if api_key.trim().starts_with("tp-") => {
            "https://token-plan-cn.xiaomimimo.com/v1".to_string()
        }
        PROVIDER_XIAOMI_MIMO => "https://api.xiaomimimo.com/v1".to_string(),
        PROVIDER_KIMI => "https://api.moonshot.cn/v1".to_string(),
        PROVIDER_GLM => "https://open.bigmodel.cn/api/paas/v4".to_string(),
        PROVIDER_MINIMAX => "https://api.minimaxi.com/v1".to_string(),
        PROVIDER_QWEN => "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
        _ => String::new(),
    }
}

fn effective_input_base_url(settings: &AppSettings, api_key: &str) -> String {
    let configured = settings.input_ai_base_url.trim();
    if !configured.is_empty() {
        return configured.to_string();
    }

    match input_provider_id(settings) {
        PROVIDER_DEEPSEEK => "https://api.deepseek.com".to_string(),
        PROVIDER_XIAOMI_MIMO if api_key.trim().starts_with("tp-") => {
            "https://token-plan-cn.xiaomimimo.com/v1".to_string()
        }
        PROVIDER_XIAOMI_MIMO => "https://api.xiaomimimo.com/v1".to_string(),
        PROVIDER_KIMI => "https://api.moonshot.cn/v1".to_string(),
        PROVIDER_GLM => "https://open.bigmodel.cn/api/paas/v4".to_string(),
        PROVIDER_MINIMAX => "https://api.minimaxi.com/v1".to_string(),
        PROVIDER_QWEN => "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
        _ => String::new(),
    }
}

fn effective_text_model(settings: &AppSettings) -> Result<String, String> {
    let configured = settings.text_ai_model.trim();
    if !configured.is_empty() {
        return Ok(configured.to_string());
    }

    match text_provider_id(settings) {
        PROVIDER_DEEPSEEK => Ok("deepseek-v4-flash".to_string()),
        PROVIDER_XIAOMI_MIMO => Ok("mimo-v2.5".to_string()),
        PROVIDER_KIMI => Ok("kimi-k2.6".to_string()),
        PROVIDER_GLM => Ok("glm-5.2".to_string()),
        PROVIDER_MINIMAX => Ok("MiniMax-M2.7".to_string()),
        PROVIDER_QWEN => Ok("qwen-plus".to_string()),
        _ => Err("文本模型未配置".to_string()),
    }
}

fn effective_vision_model(settings: &AppSettings) -> Result<String, String> {
    let configured = settings.vision_ai_model.trim();
    if !configured.is_empty() {
        return Ok(configured.to_string());
    }

    match vision_provider_id(settings) {
        PROVIDER_XIAOMI_MIMO => Ok("mimo-v2.5".to_string()),
        PROVIDER_KIMI => Ok("kimi-k2.6".to_string()),
        PROVIDER_GLM => Ok("glm-4.5v".to_string()),
        PROVIDER_MINIMAX => Ok("MiniMax-VL-01".to_string()),
        PROVIDER_QWEN => Ok("qwen3-vl-plus".to_string()),
        _ => Err(
            "视觉模型未配置；DeepSeek 当前只用于文本模型，请选择其他多模态视觉供应商".to_string(),
        ),
    }
}

fn effective_input_model(settings: &AppSettings) -> Result<String, String> {
    let configured = settings.input_ai_model.trim();
    if !configured.is_empty() {
        return Ok(configured.to_string());
    }

    match input_provider_id(settings) {
        PROVIDER_DEEPSEEK => Ok("deepseek-v4-flash".to_string()),
        PROVIDER_XIAOMI_MIMO => Ok("mimo-v2.5".to_string()),
        PROVIDER_KIMI => Ok("kimi-k2.6".to_string()),
        PROVIDER_GLM => Ok("glm-5.2".to_string()),
        PROVIDER_MINIMAX => Ok("MiniMax-M2.7".to_string()),
        PROVIDER_QWEN => Ok("qwen-plus".to_string()),
        _ => Err("输入模型未配置".to_string()),
    }
}

fn text_provider_id(settings: &AppSettings) -> &str {
    match settings.text_ai_provider.trim() {
        PROVIDER_DEEPSEEK => PROVIDER_DEEPSEEK,
        PROVIDER_XIAOMI_MIMO => PROVIDER_XIAOMI_MIMO,
        PROVIDER_KIMI => PROVIDER_KIMI,
        PROVIDER_GLM => PROVIDER_GLM,
        PROVIDER_MINIMAX => PROVIDER_MINIMAX,
        PROVIDER_QWEN => PROVIDER_QWEN,
        _ => PROVIDER_OPENAI_COMPATIBLE,
    }
}

fn vision_provider_id(settings: &AppSettings) -> &str {
    match settings.vision_ai_provider.trim() {
        PROVIDER_XIAOMI_MIMO => PROVIDER_XIAOMI_MIMO,
        PROVIDER_KIMI => PROVIDER_KIMI,
        PROVIDER_GLM => PROVIDER_GLM,
        PROVIDER_MINIMAX => PROVIDER_MINIMAX,
        PROVIDER_QWEN => PROVIDER_QWEN,
        _ => PROVIDER_OPENAI_COMPATIBLE,
    }
}

fn input_provider_id(settings: &AppSettings) -> &str {
    match settings.input_ai_provider.trim() {
        PROVIDER_DEEPSEEK => PROVIDER_DEEPSEEK,
        PROVIDER_XIAOMI_MIMO => PROVIDER_XIAOMI_MIMO,
        PROVIDER_KIMI => PROVIDER_KIMI,
        PROVIDER_GLM => PROVIDER_GLM,
        PROVIDER_MINIMAX => PROVIDER_MINIMAX,
        PROVIDER_QWEN => PROVIDER_QWEN,
        _ => PROVIDER_OPENAI_COMPATIBLE,
    }
}

fn map_request_error(error: reqwest::Error) -> String {
    if error.is_timeout() {
        "AI 请求超时，请稍后重试或调大请求超时".to_string()
    } else if error.is_connect() {
        "无法连接 AI 服务，请检查 Base URL 和网络连接".to_string()
    } else {
        "AI 请求失败，请检查网络、Base URL 和模型配置".to_string()
    }
}

fn map_http_status(status: u16) -> String {
    match status {
        400 => "AI 请求被服务拒绝，请检查模型名称和服务兼容性".to_string(),
        401 | 403 => "AI 鉴权失败，请检查 API Key 是否有效".to_string(),
        404 => "AI 接口不存在，请确认 Base URL 是否包含正确的 /v1 路径".to_string(),
        408 | 504 => "AI 服务响应超时，请稍后重试".to_string(),
        429 => "AI 服务限流或余额不足，请稍后重试或检查账户状态".to_string(),
        500..=599 => "AI 服务暂时不可用，请稍后重试".to_string(),
        _ => format!("AI 请求失败，HTTP 状态码 {status}"),
    }
}

fn map_image_http_status(status: u16) -> String {
    if status == 400 {
        return "视觉模型拒绝了截图请求，请检查视觉模型供应商、Base URL、模型名称和 API Key 是否匹配。".to_string();
    }

    map_http_status(status)
}

pub fn translation_language_label(code: &str) -> &'static str {
    match code {
        "auto" => "自动检测的语言",
        "zh-Hant" => "繁体中文",
        "en" => "英文",
        "ja" => "日文",
        "ko" => "韩文",
        "fr" => "法文",
        "de" => "德文",
        "es" => "西班牙文",
        _ => "简体中文",
    }
}

fn translation_source_instruction(code: &str) -> String {
    if code == "auto" {
        "自动识别原文语言".to_string()
    } else {
        format!("将原文按{}理解", translation_language_label(code))
    }
}
