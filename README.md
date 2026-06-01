# Wesk

Десктопный VPN-клиент для Windows. Импорт ключей VLESS / VMess / Trojan и др., системный туннель через **sing-box**, **split tunnel** — Госуслуги и российские сайты идут напрямую, остальное через VPN.

![Скриншот Wesk](screenshot.png)



## Возможности

- Импорт ключа из буфера или текста
- Импорт подписки по URL
- Подключение и отключение в один клик
- Список серверов: переименование, удаление
- Split tunnel — правила по доменам и приложениям
- Тёмный интерфейс с плавной анимацией подключения

## Быстрый старт

### Шаг 1 — окружение

| Нужно | Где взять |
|-------|-----------|
| Node.js 18+ | [nodejs.org](https://nodejs.org/) |
| Rust | [rust-lang.org/tools/install](https://www.rust-lang.org/tools/install) |
| Tauri (Windows) | [tauri.app/start/prerequisites](https://tauri.app/start/prerequisites/) |

### Шаг 2 — sing-box

Файл **не в git** — скачай вручную:

1. [Релизы sing-box](https://github.com/SagerNet/sing-box/releases) → `windows-amd64.zip`
2. Переименуй `sing-box.exe` → `sing-box-x86_64-pc-windows-msvc.exe`
3. Положи в `src-tauri/binaries/`

Автоустановка (PowerShell **от администратора**):

```powershell
powershell -ExecutionPolicy Bypass -File scripts/install-singbox.ps1
```

### Шаг 3 — запуск

Терминал **от администратора** (TUN требует прав):

```bash
npm install
npm run tauri:dev
```

### Шаг 4 — установщик (опционально)

```bash
npm run tauri:build
```

Результат: `src-tauri/target/release/bundle/`

## Ключи

| Протокол | Формат |
|----------|--------|
| VLESS | `vless://...` |
| VMess | `vmess://...` |
| Trojan | `trojan://...` |
| Shadowsocks | `ss://...` |
| SOCKS | `socks://...` |
| Hysteria2 | `hysteria2://...` |

Не поддерживается: `happ://crypto...`

## Стек

React · TypeScript · Vite · Rust · Tauri 2 · sing-box

## Структура

```
src/              React UI
src-tauri/        Rust + Tauri
  src/proxy/      ключи, sing-box
  split_tunnel/   маршрутизация
  binaries/       sing-box.exe (скачивается отдельно)
```

## Лицензия

MIT — [LICENSE](LICENSE)
