
#include "Hook.h"

//using namespace std;

HHOOK keyboardHook;
HHOOK mouseHook;
HWND WindowHwnd;
POINT p;
WCHAR task_bar_flag[100];
CWinVersionHelper cwvh;
double scale = GetScale();

LRESULT CALLBACK KeyboardProc(
    _In_ int code,
    _In_ WPARAM wParam,
    _In_ LPARAM lParam)
{

    ::MessageBoxA(NULL, "按键了", "caption", 0x00000002L);
    return CallNextHookEx(keyboardHook, code, wParam, lParam); //第一个参数一般可以为NULL
}


void SetKeyboardHook(DWORD threadId) //需要hook的线程ID
{
    // hook其他进程中的线程
     //keyboardHook= SetWindowsHookEx(WH_KEYBOARD_LL, KeyboardProc, GetModuleHandle("mydll.dll"), threadId);
    //"mydll.dll"里实现了KeyboardProc                                                                                        // hook自身进程中的线程
    keyboardHook = SetWindowsHookEx(WH_KEYBOARD_LL, KeyboardProc, NULL, threadId);
}


LRESULT CALLBACK MouseProc(
    _In_ int code,
    _In_ WPARAM wParam,
    _In_ LPARAM lParam)
{
    switch (wParam)
    {
        case WM_MOUSEWHEEL:
        {
          
            MSLLHOOKSTRUCT* info = reinterpret_cast<MSLLHOOKSTRUCT*>(lParam);
            bool down = false;

            WCHAR winclassname[100];
            p.x = info->pt.x / scale;
            p.y = info->pt.y / scale;
            WindowHwnd = WindowFromPoint(p);
            GetClassName(WindowHwnd, winclassname, 100);
            //printf("%d,%d\n", info->pt.x, info->pt.y);
            //wprintf(L"类名：%s\n",winclassname);
            //wcout << "类名：" << winclassname << endl;
            down = static_cast<std::make_signed_t<WORD>>(HIWORD(info->mouseData)) < 0;
           

            if (down && _tcscmp(winclassname, task_bar_flag)==0) {
                PostMessage(WindowHwnd, WM_APPCOMMAND, 0, APPCOMMAND_VOLUME_DOWN * 0x10000);
              
            }
            else if (_tcscmp(winclassname, task_bar_flag) == 0) {
                PostMessage(WindowHwnd, WM_APPCOMMAND, 0, APPCOMMAND_VOLUME_UP * 0x10000);
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
    // hook自身进程中的线程
    if (cwvh.IsWindows10())  wcscpy_s(task_bar_flag, L"MSTaskListWClass");
    if (cwvh.IsWindows11OrLater())  wcscpy_s(task_bar_flag, L"ReBarWindow32");

    mouseHook = SetWindowsHookEx(WH_MOUSE_LL, MouseProc, NULL, threadId);
}



double GetScale() 
{
    // 获取窗口当前显示的监视器
    // 使用桌面的句柄.
    //HWND hWnd = GetDesktopWindow();
    HMONITOR hMonitor = MonitorFromWindow(NULL, MONITOR_DEFAULTTONEAREST);

    // 获取监视器逻辑宽度与高度
    MONITORINFOEX miex;
    miex.cbSize = sizeof(miex);
    GetMonitorInfo(hMonitor, &miex);
    int cxLogical = (miex.rcMonitor.right - miex.rcMonitor.left);
    int cyLogical = (miex.rcMonitor.bottom - miex.rcMonitor.top);

    // 获取监视器物理宽度与高度
    DEVMODE dm;
    dm.dmSize = sizeof(dm);
    dm.dmDriverExtra = 0;
    EnumDisplaySettings(miex.szDevice, ENUM_CURRENT_SETTINGS, &dm);
    int cxPhysical = dm.dmPelsWidth;
    int cyPhysical = dm.dmPelsHeight;
    // 缩放比例计算  实际上使用任何一个即可
    double horzScale = ((double)cxPhysical / (double)cxLogical);
    double vertScale = ((double)cyPhysical / (double)cyLogical);
    //printf("%lf  %lf\n", horzScale, vertScale);
    
    return horzScale;
}