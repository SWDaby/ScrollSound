// ScrollSound.cpp : 定义应用程序的入口点。
//editbin /SUBSYSTEM:CONSOLE "$(OUTDIR)\$(ProjectName).exe"

#include "framework.h"
#include "ScrollSound.h"
#include "Hook.h"
#include "WIC.h"
#include "AutoRunning.h"
#include "WinVersionHelper.h"
#include <thread>
#include "Privilege.h"

#define MAX_LOADSTRING 100
#define MAX_HOOKTIME 10							//运行后自动hook的最大次数

// 全局变量:
HINSTANCE hInst;                                // 当前实例
TCHAR szTitle[MAX_LOADSTRING];                  // 标题栏文本
TCHAR szWindowClass[MAX_LOADSTRING];            // 主窗口类名
AutoRunning aR;                                 // 开机自启对象
HWND hWnd;
NOTIFYICONDATA nid;
HMENU hMenu, subMenu;							//托盘菜单
POINT pt;
INT menuItemId;

int hookTime=0;									//hook次数
bool isPause = TRUE;							//


int APIENTRY wWinMain(_In_ HINSTANCE hInstance,
	_In_opt_ HINSTANCE hPrevInstance,
	_In_ LPWSTR    lpCmdLine,
	_In_ int       nCmdShow)
{
	if (IsSettingAdmin() && !IsAdmin()) {
		adminrun();
		return FALSE;
	}
	UNREFERENCED_PARAMETER(hPrevInstance);
	UNREFERENCED_PARAMETER(lpCmdLine);

	// 初始化全局字符串
	LoadStringW(hInstance, IDS_APP_TITLE, szTitle, MAX_LOADSTRING);
	LoadStringW(hInstance, IDC_SCROLLSOUND, szWindowClass, MAX_LOADSTRING);
	MyRegisterClass(hInstance);


	if (!InitInstance(hInstance, nCmdShow))
	{
		return FALSE;
	}

	HACCEL hAccelTable = LoadAccelerators(hInstance, MAKEINTRESOURCE(IDC_SCROLLSOUND));

	MSG msg;

	//开始Hook
	SetMouseHook(0);

	//设置定时器，一分钟执行一次hook，执行10次。
	//目的是为让MousHOOK保持在HOOK链的链头，以避免和其他软件冲突。
	UINT_PTR timerId = SetTimer(NULL, 0, 1000*60, TimerProc); // 非阻塞定时器
	if (!timerId) {
		//std::cerr << "Failed to create timer." << std::endl;
	}
	

	// 主消息循环:
	while (GetMessage(&msg, nullptr, 0, 0))
	{

		if (!TranslateAccelerator(msg.hwnd, hAccelTable, &msg))
		{
			TranslateMessage(&msg);
			DispatchMessage(&msg);
		}

	}
	return (int)msg.wParam;
}



//
//  函数: MyRegisterClass()
//
//  目标: 注册窗口类。
//
ATOM MyRegisterClass(HINSTANCE hInstance)
{
	WNDCLASSEXW wcex;

	wcex.cbSize = sizeof(WNDCLASSEX);

	wcex.style = CS_HREDRAW | CS_VREDRAW;
	wcex.lpfnWndProc = WndProc;
	wcex.cbClsExtra = 0;
	wcex.cbWndExtra = 0;
	wcex.hInstance = hInstance;
	wcex.hIcon = LoadIcon(hInstance, MAKEINTRESOURCE(IDR_MAINFRAME));
	wcex.hCursor = LoadCursor(nullptr, IDC_ARROW);
	wcex.hbrBackground = (HBRUSH)(COLOR_WINDOW + 1);
	wcex.lpszMenuName = MAKEINTRESOURCEW(IDC_SCROLLSOUND);
	wcex.lpszClassName = szWindowClass;
	wcex.hIconSm = LoadIcon(wcex.hInstance, MAKEINTRESOURCE(IDR_MAINFRAME));

	return RegisterClassExW(&wcex);
}

//
//   函数: InitInstance(HINSTANCE, int)
//
//   目标: 保存实例句柄并创建主窗口
//
//   注释:
//
//        在此函数中，我们在全局变量中保存实例句柄并
//        创建和显示主程序窗口。
//
BOOL InitInstance(HINSTANCE hInstance, int nCmdShow)
{
	hInst = hInstance; // 将实例句柄存储在全局变量中
	HWND hWnd = CreateWindowEx(WS_EX_TOOLWINDOW ,
		szWindowClass, szTitle, WS_POPUP, 0, 0, 0, 0, NULL, NULL, hInstance, nullptr);


	if (!hWnd) {
		return FALSE;
	}

	// 初始化托盘和菜单
	InitTray(hInstance, hWnd);

	ShowWindow(hWnd, nCmdShow);//SW_HIDE
	UpdateWindow(hWnd);

	return TRUE;
}

//
//  函数: WndProc(HWND, UINT, WPARAM, LPARAM)
//
//  目标: 处理主窗口的消息。
//
//  WM_DESTROY  - 发送退出消息并返回
//
LRESULT CALLBACK WndProc(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam)
{
	switch (message)
	{
	case WM_DESTROY:
		Shell_NotifyIcon(NIM_DELETE, &nid);
		PostQuitMessage(0);
		break;
	case WM_USER:
		if (lParam == WM_RBUTTONDOWN)
		{
			GetCursorPos(&pt);//取鼠标坐标
			::SetForegroundWindow(hWnd);//解决在菜单外单击左键菜单不消失的问题
			menuItemId = ::TrackPopupMenu(subMenu, TPM_RETURNCMD |TPM_BOTTOMALIGN | TPM_LEFTALIGN, pt.x, pt.y, NULL, hWnd, NULL);//显示菜单并获取选项ID
			TrayMenuMessage(menuItemId);
			if (menuItemId == 0) PostMessage(hWnd, WM_LBUTTONDOWN, NULL, NULL);
		}
		break;

	default:
		return DefWindowProc(hWnd, message, wParam, lParam);
	}
	return 0;
}


// 初始化托盘和菜单
void InitTray(HINSTANCE hInstance, HWND hWnd)
{
	nid.cbSize = sizeof(nid);
	nid.hWnd = hWnd;
	nid.uID = 0;
	nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
	nid.uCallbackMessage = WM_USER;
	nid.hIcon = LoadIcon(hInst, MAKEINTRESOURCE(IDI_TRAY_ICON));
	lstrcpy(nid.szTip, APP_NAME);


	hMenu = LoadMenu(hInst, MAKEINTRESOURCE(IDR_TASK_BAR_MENU));//加载资源生成菜单

	subMenu = GetSubMenu(hMenu, 0);

	CMenuIcon::AddIconToMenuItem(subMenu, ID_ABOUT, FALSE, GetMenuIcon(IDI_TRAY_ICON));
	CMenuIcon::AddIconToMenuItem(subMenu, ID_APP_EXIT, FALSE, GetMenuIcon(IDI_EXIT_ICON));

	CheckMenuItem(subMenu, ID_AUTO_RUNNING, (aR.IsAutoRunning() ? MF_CHECKED : MF_UNCHECKED));
	CheckMenuItem(subMenu, ID_ADMIN, (IsSettingAdmin() ? MF_CHECKED : MF_UNCHECKED));
	Shell_NotifyIcon(NIM_ADD, &nid);
}

// 托盘菜单点击消息处理程序。
void TrayMenuMessage(int MessageID) {
	switch (MessageID)
	{
	case ID_APP_EXIT:
		Shell_NotifyIcon(NIM_DELETE, &nid);
		MoveMouseHook();
		PostQuitMessage(0);
		break;

	case ID_AUTO_RUNNING:
		if (aR.IsAutoRunning()) {
			aR.CanclePowerOn();
		}
		else
		{
			aR.AutoStart();
		}
		CheckMenuItem(subMenu, ID_AUTO_RUNNING, (aR.IsAutoRunning() ? MF_CHECKED : MF_UNCHECKED));
		break;

	case ID_ADMIN:
		if (IsSettingAdmin()) {
			WritePrivateProfileString(SECTION_NAME, KEY_NAME, _T("0"), SETTING_PATH);
			CheckMenuItem(subMenu, ID_ADMIN, MF_UNCHECKED);
		}
		else {
			WritePrivateProfileString(SECTION_NAME, KEY_NAME, _T("1"), SETTING_PATH);
			CheckMenuItem(subMenu, ID_ADMIN, MF_CHECKED);
			if (!IsAdmin()) {
				adminrun();
				Shell_NotifyIcon(NIM_DELETE, &nid);
				PostQuitMessage(0);
			}
		}
		break;

	case ID_ABOUT:
		ShellExecute(NULL, _T("open"), _T("https://github.com/SWDaby/ScrollSound"), NULL, NULL, SW_SHOW);
		break;

	case ID_REHOOK:
		//cout << "ReHook" << endl;
		ReHook();
		MessageBoxTimeout(NULL, _T("HOOK 成功"), _T("REHOOK"), MB_ICONINFORMATION, 1000);
		ModifyMenu(subMenu, ID_PAUSE, MF_BYCOMMAND | MF_STRING, ID_PAUSE, _T("暂停"));
		isPause = TRUE;
		break;

	case ID_PAUSE:
		if (isPause) {
			MoveMouseHook();
			//cout << "MoveMouseHook" << endl;
			ModifyMenu(subMenu, ID_PAUSE, MF_BYCOMMAND | MF_STRING, ID_PAUSE, _T("继续"));
			isPause = FALSE;
		}
		else
		{
			SetMouseHook(0);
			//cout << "SetMouseHook" << endl;
			ModifyMenu(subMenu, ID_PAUSE, MF_BYCOMMAND | MF_STRING, ID_PAUSE, _T("暂停"));
			isPause = TRUE;
		}
		break;
	}

}



//定时器回调
void CALLBACK TimerProc(HWND hwnd, UINT uMsg, UINT_PTR idEvent, DWORD dwTime) {
	//std::cout << "Timer triggered at " << dwTime << " ms!" << std::endl;
	if (isPause) {
		ReHook();
	}
	hookTime++;
	if (hookTime == MAX_HOOKTIME) {
		KillTimer(NULL, idEvent);
	}
	
}

int MessageBoxTimeout(HWND hWnd, LPCTSTR lpText, LPCTSTR lpCaption, UINT uType, DWORD dwMilliseconds) {
	std::thread([=]() {
		std::this_thread::sleep_for(std::chrono::milliseconds(dwMilliseconds));
		HWND hMsgBox = FindWindow(NULL, lpCaption);
		if (hMsgBox) {
			PostMessage(hMsgBox, WM_CLOSE, 0, 0);
		}
		}).detach();

	return MessageBox(hWnd, lpText, lpCaption, uType);
}