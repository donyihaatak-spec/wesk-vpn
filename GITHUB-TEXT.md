# Тексты для GitHub

Скопируй нужные блоки при создании репозитория.

---

## Название репозитория

```
wesk-vpn
```

или

```
wesk
```

---

## Описание (About → Description, до 350 символов)

```
Desktop VPN client for Windows. Tauri 2 + React + sing-box TUN. VLESS/VMess/Trojan import, split tunnel, smooth UI.
```

**На русском (если репо для Kwork/портфолио):**

```
Wesk — десктопный VPN-клиент для Windows (Tauri 2 + React + sing-box). Импорт ключей, split tunnel, Госуслуги мимо VPN.
```

---

## Topics (теги)

```
vpn
tauri
react
rust
sing-box
vless
split-tunnel
windows
desktop-app
typescript
```

---

## Website (опционально)

Оставь пустым или укажи ссылку на Kwork / портфолио.

---

## Первый commit message

```
Initial commit: Wesk VPN client for Windows
```

---

## Release notes (если будешь делать Release v0.1.0)

```markdown
## Wesk v0.1.0

Первый публичный релиз исходников.

### Возможности
- Импорт VLESS, VMess, Trojan, Shadowsocks, SOCKS, Hysteria2
- Подписки по URL
- sing-box TUN на Windows
- Split tunnel (Госуслуги, Telegram и др.)
- Тёмный UI с плавными переходами подключения

### Сборка
```bash
npm install
# sing-box → src-tauri/binaries/
npm run tauri:dev
```
```

---

## README

Файл `README.md` уже в архиве — менять не обязательно.  
Добавь `screenshot.png` в корень и в README замени блок «Скриншот» на:

```markdown
![Wesk](screenshot.png)
```
