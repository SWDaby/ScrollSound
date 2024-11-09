#include "MonitorHelper.h"
#include "framework.h"
double MonitorHelper::GetScale() const
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
