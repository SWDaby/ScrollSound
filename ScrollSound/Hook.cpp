#include "Hook.h"

HHOOK mouseHook;
HWND windowHwnd;
POINT p;
TCHAR taskBarClassName[100];
CWinVersionHelper cwvh;

MSLLHOOKSTRUCT* mWheelInfo;	
bool down = false;			//滚轮是否向下滚动

MSLLHOOKSTRUCT* mButtonDownInfo;
TCHAR winClassName[100];

LRESULT CALLBACK MouseProc(
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
		mWheelInfo = reinterpret_cast<MSLLHOOKSTRUCT*>(lParam);

		//std::wcout << "类名：" << winClassName << endl;
		down = static_cast<std::make_signed_t<WORD>>(HIWORD(mWheelInfo->mouseData)) < 0;
		if (down && _tcscmp(winClassName, taskBarClassName) == 0) {
			PostMessage(windowHwnd, WM_APPCOMMAND, 0, APPCOMMAND_VOLUME_DOWN << 16);

		}
		else if (_tcscmp(winClassName, taskBarClassName) == 0) {
			PostMessage(windowHwnd, WM_APPCOMMAND, 0, APPCOMMAND_VOLUME_UP << 16);
		}
		break;
	}


	case WM_MBUTTONDOWN:
	{
		GetCursorPos(&p);
		windowHwnd = WindowFromPoint(p);
		GetClassName(windowHwnd, winClassName, 100);

		//解决和其他的全局钩子冲突,如：会和wgesture冲突，触发两次WM_MBUTTONDOWN
		mButtonDownInfo = reinterpret_cast<MSLLHOOKSTRUCT*>(lParam);
		if (mButtonDownInfo) {
			//std::cout << "flags: " << mButtonDownInfo->flags << std::endl;
			//std::cout << "dwExtraInfo: " << mButtonDownInfo->dwExtraInfo << std::endl;
			if (mButtonDownInfo->dwExtraInfo == 0 && _tcscmp(winClassName, taskBarClassName) == 0) {
				PostMessage(windowHwnd, WM_APPCOMMAND, 0, APPCOMMAND_VOLUME_MUTE << 16);
				break;
			}		
		}

	}

	}


	return CallNextHookEx(NULL, code, wParam, lParam); //第一个参数一般可以为NULL
}


//**************************************************
//	函数：SetMouseHook()
//
//	功能： 监听全局鼠标
//
//	参数：threadId为需要hook的线程ID,0为hook全局；
//
//	返回值：void
// SetWindowsHookEx 函数会将一个钩子安装在钩子链的头部。
// 当被某种钩子监视的消息出现时，系统会从钩子链的链头开始调用与该类型钩子关联的钩子回调方法。
// 钩子链中的每个钩子决定了是否将消息传递给下一个过程。
// 通过调用 CallNextHookEx，钩子过程将事件传递给下一个过程。
//**************************************************
void SetMouseHook(DWORD threadId)
{
	if (cwvh.IsWindows10()) {
		_tcscpy_s(taskBarClassName, _T("MSTaskListWClass"));
	}

	if (cwvh.IsWindows11_21H2()) {
		_tcscpy_s(taskBarClassName, _T("ReBarWindow32"));
	}

	if (cwvh.IsWindows11_22H2OrLater()) {
		_tcscpy_s(taskBarClassName, _T("Shell_TrayWnd"));
	}

	mouseHook = SetWindowsHookEx(WH_MOUSE_LL, MouseProc, NULL, threadId);
}

void MoveMouseHook() {
	UnhookWindowsHookEx(mouseHook);
	mouseHook = NULL;
}

void ReHook() {
	MoveMouseHook();
	SetMouseHook(0);
}