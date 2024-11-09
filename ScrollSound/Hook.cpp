#include "Hook.h"

HHOOK mouseHook;
HWND windowHwnd;
POINT p;
WCHAR taskBarClassName[100];
CWinVersionHelper cwvh;

MSLLHOOKSTRUCT* info;
TCHAR winClassName[100];
short i = 0;


LRESULT CALLBACK Win10MouseProc(
	_In_ int code,
	_In_ WPARAM wParam,
	_In_ LPARAM lParam)
{


	switch (wParam)
	{
	case WM_MOUSEWHEEL:
	{
		GetCursorPos(&p);
		windowHwnd = WindowFromPoint(p);
		GetClassName(windowHwnd, winClassName, 100);
		info = reinterpret_cast<MSLLHOOKSTRUCT*>(lParam);
		bool down = false;
		//wcout << "类名：" << winClassName << endl;
		down = static_cast<std::make_signed_t<WORD>>(HIWORD(info->mouseData)) < 0;

		if (down && _tcscmp(winClassName, taskBarClassName) == 0) {
			PostMessage(windowHwnd, WM_APPCOMMAND, 0, APPCOMMAND_VOLUME_DOWN * 0x10000);

		}
		else if (_tcscmp(winClassName, taskBarClassName) == 0) {
			PostMessage(windowHwnd, WM_APPCOMMAND, 0, APPCOMMAND_VOLUME_UP * 0x10000);
		}

		break;
	}

	case WM_MBUTTONDOWN: {
		GetCursorPos(&p);
		windowHwnd = WindowFromPoint(p);
		GetClassName(windowHwnd, winClassName, 100);
		i++;
		if (i == 2 && _tcscmp(winClassName, taskBarClassName) == 0) {
			PostMessage(windowHwnd, WM_APPCOMMAND, 0, APPCOMMAND_VOLUME_MUTE << 16);
			i = 0;
		}
		else if (i >= 2) {
			i = 0;
		}
		//wcout << i << endl;
		break;

	}
	}


	return CallNextHookEx(mouseHook, code, wParam, lParam); //第一个参数一般可以为NULL
}

LRESULT CALLBACK Win11MouseProc(
	_In_ int code,
	_In_ WPARAM wParam,
	_In_ LPARAM lParam)
{


	switch (wParam)
	{
	case WM_MOUSEWHEEL:
	{

		GetCursorPos(&p);
		windowHwnd = WindowFromPoint(p);
		GetClassName(windowHwnd, winClassName, 100);
		info = reinterpret_cast<MSLLHOOKSTRUCT*>(lParam);
		bool down = false;
		//wcout << "类名：" << winClassName << endl;
		down = static_cast<std::make_signed_t<WORD>>(HIWORD(info->mouseData)) < 0;

		if (down && _tcscmp(winClassName, taskBarClassName) == 0) {
			PostMessage(windowHwnd, WM_APPCOMMAND, 0, APPCOMMAND_VOLUME_DOWN * 0x10000);

		}
		else if (_tcscmp(winClassName, taskBarClassName) == 0) {
			PostMessage(windowHwnd, WM_APPCOMMAND, 0, APPCOMMAND_VOLUME_UP * 0x10000);
		}

		break;
	}

	case WM_MBUTTONDOWN: {
		GetCursorPos(&p);
		windowHwnd = WindowFromPoint(p);
		GetClassName(windowHwnd, winClassName, 100);
		if (_tcscmp(winClassName, taskBarClassName) == 0) {
			PostMessage(windowHwnd, WM_APPCOMMAND, 0, APPCOMMAND_VOLUME_MUTE << 16);			
		}
		break;

	}

	}


	return CallNextHookEx(mouseHook, code, wParam, lParam); //第一个参数一般可以为NULL
}
 

//**************************************************
//	函数：SetMouseHook()
//
//	功能： 监听全局鼠标
//
//	参数：threadId为需要hook的线程ID,0为hook全局；
//
//	返回值：void
//**************************************************
void SetMouseHook(DWORD threadId)
{
	if (cwvh.IsWindows10()) {
		wcscpy_s(taskBarClassName, L"MSTaskListWClass");
		mouseHook = SetWindowsHookEx(WH_MOUSE_LL, Win10MouseProc, NULL, threadId);
	}

	if (cwvh.IsWindows11_21H2()) {
		wcscpy_s(taskBarClassName, L"ReBarWindow32");
		mouseHook = SetWindowsHookEx(WH_MOUSE_LL, Win11MouseProc, NULL, threadId);

	} 

	if (cwvh.IsWindows11_22H2OrLater()) {
		wcscpy_s(taskBarClassName, L"Shell_TrayWnd");
		mouseHook = SetWindowsHookEx(WH_MOUSE_LL, Win11MouseProc, NULL, threadId);

	}

	//mouseHook = SetWindowsHookEx(WH_MOUSE_LL, Win10MouseProc, NULL, threadId);
}

