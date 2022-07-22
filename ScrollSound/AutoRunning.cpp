
#include "AutoRunning.h"


// 键：计算机\HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run
//#define APP_NAME "WindowsProject1"
AutoRunning::AutoRunning()
{
    GetModuleFileName(NULL, strExeFullDir, MAX_PATH);
    RegOpenKeyEx(HKEY_CURRENT_USER, _T("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run"), 0, KEY_ALL_ACCESS, &hKey);

}

AutoRunning::~AutoRunning()
{
    
}

void AutoRunning:: AutoStart()
{

        //已经存在
        if (!IsAutoRunning())
        {
            //添加一个子Key,并设置值，"APP_NAME"是应用程序名字（不加后缀.exe） 
            RegSetValueEx(hKey, APP_NAME, 0, REG_SZ, (LPBYTE)strExeFullDir, (lstrlen(strExeFullDir) + 1) * sizeof(TCHAR));
            RegCloseKey(hKey);
           
        }
    
}


//取消当前程序开机启动
void AutoRunning:: CanclePowerOn()
{

   
    if (IsAutoRunning())
    {
        //删除值
        RegDeleteValue(hKey, APP_NAME);

        //关闭注册表
        RegCloseKey(hKey);
    }
}


bool AutoRunning:: IsAutoRunning()
{
    if (RegOpenKeyEx(HKEY_CURRENT_USER, _T("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run"), 0, KEY_ALL_ACCESS, &hKey) == ERROR_SUCCESS) ///打开启动项       
   {

        TCHAR strDir[MAX_PATH] = {};
        DWORD nLength = MAX_PATH;
        long result = RegGetValue(hKey, nullptr, APP_NAME, RRF_RT_REG_SZ, 0, strDir, &nLength);
        if (result != ERROR_SUCCESS || _tcscmp(strExeFullDir, strDir) != 0)
            return false;
        else return true;
    }
    return false;
}


