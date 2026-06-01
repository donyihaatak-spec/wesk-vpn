//! Единый тип ошибок приложения.
//!
//! Tauri-команды возвращают `Result<T, E>`, где `E` обязан реализовывать
//! `serde::Serialize`, чтобы ошибка могла пересечь IPC-границу и быть
//! обработанной во фронтенде. Поэтому ниже реализована ручная сериализация
//! в строку с человекочитаемым сообщением.

use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Ошибка ввода-вывода: {0}")]
    Io(#[from] std::io::Error),

    #[error("Ошибка сериализации данных: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Некорректная конфигурация WireGuard: {0}")]
    InvalidConfig(String),

    #[error("Конфигурация не найдена: {0}")]
    NotFound(String),

    #[error("Не удалось определить каталог данных приложения")]
    NoDataDir,

    #[error("Внутренняя ошибка: {0}")]
    Other(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
