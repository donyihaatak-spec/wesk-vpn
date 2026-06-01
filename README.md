# Wesk

Десктопный VPN/прокси-клиент для Windows (Tauri 2). Импорт ключей в стиле Happ, подключение через **sing-box** в режиме TUN, настройка **split tunnel** — какие приложения идут через VPN, какие напрямую.

**Идентификатор приложения:** `com.neonclick.vpnconfigurator`

## Стек

- **UI:** React 18, TypeScript, Vite
- **Бэкенд:** Rust, Tauri 2
- **Туннель:** sing-box (VLESS, VMess, Trojan, Shadowsocks, SOCKS, Hysteria2)
- **Legacy:** WireGuard `.conf` (отдельный модуль, не основной сценарий в UI)

## Возможности

- Импорт ключа из буфера или текста (`vless://`, `vmess://`, `trojan://`, `ss://`, `socks://`, `hysteria2://`)
- Импорт подписки по URL
- Подключение и отключение одним действием
- Список серверов: переименование, удаление
- Split tunnel — правила по приложениям и доменам
- Тёмный интерфейс с плавными переходами подключения/отключения

## Скриншот

![Wesk](screenshot.png)

## Быстрый старт

### Требования

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- Зависимости Tauri 2: [tauri.app/start/prerequisites](https://tauri.app/start/prerequisites/)
- **sing-box** — см. [src-tauri/binaries/README.md](src-tauri/binaries/README.md)
- Для TUN на Windows — запуск **от имени администратора**

### Запуск

```bash
npm install

# Положите sing-box в src-tauri/binaries/ (инструкция в README там)

npm run tauri:dev
```

Сборка установщика:

```bash
npm run tauri:build
```

## Поддерживаемые форматы ключей

| Протокол | Пример |
|----------|--------|
| VLESS | `vless://uuid@host:443?...` |
| VMess | `vmess://base64...` |
| Trojan | `trojan://password@host:443` |
| Shadowsocks | `ss://...` |
| SOCKS | `socks://user:pass@host:1080` |
| Hysteria2 | `hysteria2://password@host:443` |

Пока **не поддерживается:** `happ://crypto...` (зашифрованные подписки Happ).

## Структура проекта

```
├── src/                      # React UI
│   ├── App.tsx
│   ├── components/
│   ├── hooks/                # useProfiles, useDisplayStatus, …
│   ├── lib/                  # tauri.ts, smoothTransition.ts, brand.ts
│   └── styles/
│
└── src-tauri/                # Rust + Tauri
    ├── src/
    │   ├── commands.rs       # IPC: connect, import, split tunnel
    │   ├── proxy/            # парсер ключей, sing-box, менеджер
    │   ├── split_tunnel/     # правила маршрутизации
    │   └── vpn/              # WireGuard (legacy)
    └── binaries/             # sing-box.exe (не в git)
```

## Как устроено подключение

1. Пользователь импортирует ключ → Rust парсит URI (`src-tauri/src/proxy/uri.rs`).
2. По нажатию «Подключить» собирается конфиг sing-box и поднимается TUN.
3. Split tunnel добавляет правила в конфиг sing-box перед запуском.

WireGuard используется только если работать со старыми `.conf` через отдельные команды бэкенда.

## Проверка бэкенда

```bash
cd src-tauri
cargo check
cargo test
```

## Загрузка на GitHub

```powershell
npm run prepare:github
```

Появится папка **`wesk-vpn-github/`** — только клиент Wesk, без магазина. Подробно: [GITHUB.md](GITHUB.md).

## Лицензия

MIT — см. [LICENSE](LICENSE).
