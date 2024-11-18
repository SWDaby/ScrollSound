#pragma once

#include "resource.h"

#ifdef _DEBUG
#define DEBUG_PRINT(msg) std::cout << "Debug: " << msg << std::endl
#else
#define DEBUG_PRINT(msg) // 什么都不做
#endif

ATOM                MyRegisterClass(HINSTANCE hInstance);
BOOL                InitInstance(HINSTANCE, int);
LRESULT CALLBACK    WndProc(HWND, UINT, WPARAM, LPARAM);

void TrayMenuMessage(int);
void InitTray(HINSTANCE hInstance, HWND hWnd);

void CALLBACK TimerProc(HWND hwnd, UINT uMsg, UINT_PTR idEvent, DWORD dwTime);
int MessageBoxTimeout(HWND hWnd, LPCTSTR lpText, LPCTSTR lpCaption, UINT uType, DWORD dwMilliseconds);