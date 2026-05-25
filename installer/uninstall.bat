@echo off
chcp 65001 >nul
title AI filter - O'chirish

net session >nul 2>&1
if %errorlevel% neq 0 (
    echo [!] Administrator huquqi kerak. O'ng tugma - "Administrator sifatida ishga tushirish"
    pause
    exit /b 1
)

echo.
echo  AI filter dasturini o'chirishni xohlaysizmi?
set /p CONFIRM="Davom etish uchun Y bosing (Y/N): "
if /i not "%CONFIRM%"=="Y" exit /b 0

echo.
echo [1/3] Dastur to'xtatilmoqda...
taskkill /F /IM ai-filter.exe >nul 2>&1

echo [2/3] Fayllar o'chirilmoqda...
rmdir /S /Q "%ProgramFiles%\AI filter" >nul 2>&1

echo [3/3] Avtomatik ishga tushirish o'chirilmoqda...
reg delete "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v "AIfilter" /f >nul 2>&1

:: Ish stolidagi yorliqni o'chirish
del "%USERPROFILE%\Desktop\AI filter.lnk" >nul 2>&1

echo.
echo  ✅ AI filter muvaffaqiyatli o'chirildi!
echo.
pause
