
#include "AutoRunning.h"


// ���������\HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run
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

        //�Ѿ�����
        if (!IsAutoRunning())
        {
            //���һ����Key,������ֵ��"APP_NAME"��Ӧ�ó������֣����Ӻ�׺.exe�� 
            RegSetValueEx(hKey, APP_NAME, 0, REG_SZ, (LPBYTE)strExeFullDir, (lstrlen(strExeFullDir) + 1) * sizeof(TCHAR));
            RegCloseKey(hKey);
           
        }
    
}


//ȡ����ǰ���򿪻�����
void AutoRunning:: CanclePowerOn()
{

   
    if (IsAutoRunning())
    {
        //ɾ��ֵ
        RegDeleteValue(hKey, APP_NAME);

        //�ر�ע���
        RegCloseKey(hKey);
    }
}


bool AutoRunning:: IsAutoRunning()
{
    if (RegOpenKeyEx(HKEY_CURRENT_USER, _T("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run"), 0, KEY_ALL_ACCESS, &hKey) == ERROR_SUCCESS) ///��������       
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


