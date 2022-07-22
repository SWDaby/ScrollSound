
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

    ::MessageBoxA(NULL, "������", "caption", 0x00000002L);
    return CallNextHookEx(keyboardHook, code, wParam, lParam); //��һ������һ�����ΪNULL
}


void SetKeyboardHook(DWORD threadId) //��Ҫhook���߳�ID
{
    // hook���������е��߳�
     //keyboardHook= SetWindowsHookEx(WH_KEYBOARD_LL, KeyboardProc, GetModuleHandle("mydll.dll"), threadId);
    //"mydll.dll"��ʵ����KeyboardProc                                                                                        // hook��������е��߳�
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
            //wprintf(L"������%s\n",winclassname);
            //wcout << "������" << winclassname << endl;
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


    return CallNextHookEx(mouseHook, code, wParam, lParam); //��һ������һ�����ΪNULL
}


//**************************************************
//	������SetMouseHook()
//
//	���ܣ� ����ȫ�����
//
//	������threadIdΪ��Ҫhook���߳�ID,0Ϊhookȫ�֣�
//
//	����ֵ��void
//**************************************************
void SetMouseHook(DWORD threadId) 
{
    // hook��������е��߳�
    if (cwvh.IsWindows10())  wcscpy_s(task_bar_flag, L"MSTaskListWClass");
    if (cwvh.IsWindows11OrLater())  wcscpy_s(task_bar_flag, L"ReBarWindow32");

    mouseHook = SetWindowsHookEx(WH_MOUSE_LL, MouseProc, NULL, threadId);
}



double GetScale() 
{
    // ��ȡ���ڵ�ǰ��ʾ�ļ�����
    // ʹ������ľ��.
    //HWND hWnd = GetDesktopWindow();
    HMONITOR hMonitor = MonitorFromWindow(NULL, MONITOR_DEFAULTTONEAREST);

    // ��ȡ�������߼������߶�
    MONITORINFOEX miex;
    miex.cbSize = sizeof(miex);
    GetMonitorInfo(hMonitor, &miex);
    int cxLogical = (miex.rcMonitor.right - miex.rcMonitor.left);
    int cyLogical = (miex.rcMonitor.bottom - miex.rcMonitor.top);

    // ��ȡ��������������߶�
    DEVMODE dm;
    dm.dmSize = sizeof(dm);
    dm.dmDriverExtra = 0;
    EnumDisplaySettings(miex.szDevice, ENUM_CURRENT_SETTINGS, &dm);
    int cxPhysical = dm.dmPelsWidth;
    int cyPhysical = dm.dmPelsHeight;
    // ���ű�������  ʵ����ʹ���κ�һ������
    double horzScale = ((double)cxPhysical / (double)cxLogical);
    double vertScale = ((double)cyPhysical / (double)cyLogical);
    //printf("%lf  %lf\n", horzScale, vertScale);
    
    return horzScale;
}