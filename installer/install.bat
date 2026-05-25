@echo off
chcp 65001 >nul
title AI filter - O'rnatish

:: ========================================
:: Administrator huquqini tekshirish
:: ========================================
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo.
    echo [!] Bu o'rnatuvchi Administrator huquqini talab qiladi.
    echo     Iltimos, sichqonchaning o'ng tugmasini bosib "Administrator sifatida ishga tushirish" ni tanlang.
    echo.
    pause
    exit /b 1
)

echo.
echo  ╔══════════════════════════════════════════╗
echo  ║       🛡  AI filter - O'rnatuvchi        ║
echo  ║      Shaxsiy ma'lumotlar himoyachisi     ║
echo  ╚══════════════════════════════════════════╝
echo.

set "INSTALL_DIR=%ProgramFiles%\AI filter"
set "DESKTOP=%USERPROFILE%\Desktop"

:: ========================================
:: 1. Papka yaratish
:: ========================================
echo [1/4] Dastur papkasi yaratilmoqda...
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"
if not exist "%INSTALL_DIR%\ca" mkdir "%INSTALL_DIR%\ca"

:: ========================================
:: 2. Fayllarni nusxalash
:: ========================================
echo [2/4] Fayllar nusxalanmoqda...
copy /Y "%~dp0ai-filter.exe" "%INSTALL_DIR%\ai-filter.exe" >nul
copy /Y "%~dp0ca\root.crt" "%INSTALL_DIR%\ca\root.crt" >nul
copy /Y "%~dp0ca\root.key" "%INSTALL_DIR%\ca\root.key" >nul

:: ========================================
:: 3. Root CA sertifikatni o'rnatish
:: ========================================
echo [3/4] Root CA sertifikati o'rnatilmoqda (ishonchli sertifikatlar ro'yxatiga)...
certutil -addstore "Root" "%INSTALL_DIR%\ca\root.crt" >nul 2>&1
if %errorlevel% equ 0 (
    echo       ✓ Sertifikat muvaffaqiyatli o'rnatildi!
) else (
    echo       ⚠ Sertifikatni o'rnatib bo'lmadi. Qo'lda o'rnating.
)

:: ========================================
:: 4. Ish stolida yorliq yaratish
:: ========================================
echo [4/4] Ish stolida yorliq yaratilmoqda...
powershell -Command "$ws = New-Object -ComObject WScript.Shell; $s = $ws.CreateShortcut('%DESKTOP%\AI filter.lnk'); $s.TargetPath = '%INSTALL_DIR%\ai-filter.exe'; $s.WorkingDirectory = '%INSTALL_DIR%'; $s.Description = 'AI filter - shaxsiy malumotar himoyasi'; $s.Save()"

:: ========================================
:: 5. Windows ishga tushirishga qo'shish
:: ========================================
echo.
set /p STARTUP="Windows bilan birga avtomatik ishga tushsinmi? (H/Y) [H]: "
if /i "%STARTUP%"=="Y" (
    reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v "AIfilter" /t REG_SZ /d "\"%INSTALL_DIR%\ai-filter.exe\"" /f >nul
    echo       ✓ Avtomatik ishga tushirish yoqildi!
) else (
    echo       Avtomatik ishga tushirish o'tkazib yuborildi.
)

:: ========================================
:: YAKUNLASH
:: ========================================
echo.
echo  ╔══════════════════════════════════════════╗
echo  ║  ✅ AI filter muvaffaqiyatli o'rnatildi! ║
echo  ╠══════════════════════════════════════════╣
echo  ║  📂 Joylashuv: %ProgramFiles%\AI filter  ║
echo  ║  🖥  Ish stolida "AI filter" yorlig'i    ║
echo  ║  🔐 Root CA sertifikati o'rnatildi       ║
echo  ╚══════════════════════════════════════════╝
echo.
echo  Dasturni ishga tushirish uchun ish stolidagi
echo  "AI filter" yorlig'ini ikki marta bosing.
echo.
pause
