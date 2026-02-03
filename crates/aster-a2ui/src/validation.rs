//! A2UI 验证工具
//!
//! 提供 JSON Pointer 路径解析和数据模型验证功能

use serde_json::Value;

/// JSON Pointer 路径解析错误
#[derive(Debug, Clone, PartialEq)]
pub enum JsonPointerError {
    /// 路径格式无效
    InvalidFormat(String),
    /// 路径不存在
    PathNotFound(String),
    /// 数组索引无效
    InvalidArrayIndex(String),
}

impl std::fmt::Display for JsonPointerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFormat(msg) => write!(f, "无效的 JSON Pointer 格式: {}", msg),
            Self::PathNotFound(path) => write!(f, "路径不存在: {}", path),
            Self::InvalidArrayIndex(idx) => write!(f, "无效的数组索引: {}", idx),
        }
    }
}

impl std::error::Error for JsonPointerError {}

/// 解析 JSON Pointer 路径并获取值
///
/// 支持绝对路径（以 `/` 开头）和相对路径
pub fn resolve_pointer<'a>(data: &'a Value, pointer: &str) -> Result<&'a Value, JsonPointerError> {
    if pointer.is_empty() || pointer == "/" {
        return Ok(data);
    }

    // 确保路径以 / 开头
    let normalized = if pointer.starts_with('/') {
        pointer.to_string()
    } else {
        format!("/{}", pointer)
    };

    data.pointer(&normalized)
        .ok_or_else(|| JsonPointerError::PathNotFound(pointer.to_string()))
}

/// 解析 JSON Pointer 路径并获取可变引用
pub fn resolve_pointer_mut<'a>(
    data: &'a mut Value,
    pointer: &str,
) -> Result<&'a mut Value, JsonPointerError> {
    if pointer.is_empty() || pointer == "/" {
        return Ok(data);
    }

    let normalized = if pointer.starts_with('/') {
        pointer.to_string()
    } else {
        format!("/{}", pointer)
    };

    data.pointer_mut(&normalized)
        .ok_or_else(|| JsonPointerError::PathNotFound(pointer.to_string()))
}

/// 在指定路径设置值，自动创建中间路径
pub fn set_at_pointer(
    data: &mut Value,
    pointer: &str,
    value: Value,
) -> Result<(), JsonPointerError> {
    if pointer.is_empty() || pointer == "/" {
        *data = value;
        return Ok(());
    }

    let normalized = if let Some(stripped) = pointer.strip_prefix('/') {
        stripped
    } else {
        pointer
    };

    let parts: Vec<&str> = normalized.split('/').collect();
    let mut current = data;

    for (i, part) in parts.iter().enumerate() {
        let is_last = i == parts.len() - 1;

        if is_last {
            // 最后一个部分，设置值
            if let Ok(idx) = part.parse::<usize>() {
                if let Value::Array(arr) = current {
                    while arr.len() <= idx {
                        arr.push(Value::Null);
                    }
                    arr[idx] = value;
                    return Ok(());
                }
            }
            if let Value::Object(obj) = current {
                obj.insert(part.to_string(), value);
                return Ok(());
            }
            return Err(JsonPointerError::InvalidFormat(
                "父节点不是对象或数组".to_string(),
            ));
        }

        // 中间部分，导航或创建
        if let Ok(idx) = part.parse::<usize>() {
            if let Value::Array(arr) = current {
                while arr.len() <= idx {
                    arr.push(Value::Null);
                }
                if arr[idx].is_null() {
                    // 检查下一个部分是否是数字
                    let next_is_array = parts
                        .get(i + 1)
                        .map(|p| p.parse::<usize>().is_ok())
                        .unwrap_or(false);
                    arr[idx] = if next_is_array {
                        Value::Array(vec![])
                    } else {
                        Value::Object(serde_json::Map::new())
                    };
                }
                current = &mut arr[idx];
                continue;
            }
        }

        if let Value::Object(obj) = current {
            if !obj.contains_key(*part) {
                let next_is_array = parts
                    .get(i + 1)
                    .map(|p| p.parse::<usize>().is_ok())
                    .unwrap_or(false);
                obj.insert(
                    part.to_string(),
                    if next_is_array {
                        Value::Array(vec![])
                    } else {
                        Value::Object(serde_json::Map::new())
                    },
                );
            }
            current = obj.get_mut(*part).unwrap();
            continue;
        }

        return Err(JsonPointerError::InvalidFormat(format!(
            "无法在路径 {} 处导航",
            part
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_resolve_pointer() {
        let data = json!({
            "user": {
                "name": "张三",
                "age": 25
            },
            "items": ["a", "b", "c"]
        });

        assert_eq!(resolve_pointer(&data, "/user/name").unwrap(), "张三");
        assert_eq!(resolve_pointer(&data, "/user/age").unwrap(), 25);
        assert_eq!(resolve_pointer(&data, "/items/0").unwrap(), "a");
        assert!(resolve_pointer(&data, "/nonexistent").is_err());
    }

    #[test]
    fn test_set_at_pointer() {
        let mut data = json!({});

        set_at_pointer(&mut data, "/user/name", json!("李四")).unwrap();
        assert_eq!(data["user"]["name"], "李四");

        set_at_pointer(&mut data, "/items/0", json!("first")).unwrap();
        assert_eq!(data["items"][0], "first");
    }
}
