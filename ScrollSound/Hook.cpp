#include "Hook.h"

HHOOK mouseHook;
HWND windowHwnd;
POINT p;
TCHAR taskBarClassName[100];
CWinVersionHelper cwvh;

MSLLHOOKSTRUCT* mWheelInfo;	
bool down = false;			//�����Ƿ����¹���

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

		//std::wcout << "������" << winClassName << endl;
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

		//�����������ȫ�ֹ��ӳ�ͻ,�磺���wgesture��ͻ����������WM_MBUTTONDOWN
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


	return CallNextHookEx(NULL, code, wParam, lParam); //��һ������һ�����ΪNULL
}


//**************************************************
//	������SetMouseHook()
//
//	���ܣ� ����ȫ�����
//
//	������threadIdΪ��Ҫhook���߳�ID,0Ϊhookȫ�֣�
//
//	����ֵ��void
// SetWindowsHookEx �����Ὣһ�����Ӱ�װ�ڹ�������ͷ����
// ����ĳ�ֹ��Ӽ��ӵ���Ϣ����ʱ��ϵͳ��ӹ���������ͷ��ʼ����������͹��ӹ����Ĺ��ӻص�������
// �������е�ÿ�����Ӿ������Ƿ���Ϣ���ݸ���һ�����̡�
// ͨ������ CallNextHookEx�����ӹ��̽��¼����ݸ���һ�����̡�
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