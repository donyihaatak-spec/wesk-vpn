# sing-box (ядро proxy, как Xray в Happ)

Приложение использует **sing-box** для подключения по ключам `vless://`, `vmess://` и др.

## Установка (Windows)

1. Откройте https://github.com/SagerNet/sing-box/releases
2. Скачайте архив **sing-box-*-windows-amd64.zip**
3. Распакуйте и скопируйте `sing-box.exe` сюда, переименовав в:

```
sing-box-x86_64-pc-windows-msvc.exe
```

4. Итоговый путь:

```
src-tauri/binaries/sing-box-x86_64-pc-windows-msvc.exe
```

## Требования

- **Права администратора** — для TUN-режима (системный VPN)
- При первом запуске Windows может запросить UAC

## Проверка

```powershell
.\src-tauri\binaries\sing-box-x86_64-pc-windows-msvc.exe version
```

## Альтернатива

Добавьте `sing-box.exe` в PATH — приложение попробует найти его автоматически.
