#include "Privilege.h"




//获取exe路径 
wstring GetExePath()
{
    WCHAR buf[MAX_PATH] = { 0 };
    wstring strPath;
    GetModuleFileName(nullptr, buf, MAX_PATH);
    
    strPath = buf;
    return strPath;
}


void adminrun() 
{
    SHELLEXECUTEINFO sei = { sizeof(SHELLEXECUTEINFO) };
    //请求提高权限
    sei.lpVerb = TEXT("runas");
    //需要提升权限的应用程序

    wstring exePath(GetExePath());
    sei.lpFile = exePath.c_str(); //TEXT("D:\\WindowsProject1 - 副本\\x64\\Debug\\WindowsProject1.exe");

    //sei.nShow = SW_SHOWNORMAL;

    if (!ShellExecuteEx(&sei)) {
        DWORD dwStatus = GetLastError();
        if (dwStatus == ERROR_CANCELLED) {
            ::MessageBoxA(NULL, "ShellExecute Cancel..", "caption", 0x00000002L);
            //printf("ShellExecute Cancel..");
        }
        else if (dwStatus == ERROR_FILE_NOT_FOUND)
        {
            
            ::MessageBoxA(NULL, "File Not Found", "caption", 0x00000002L);
            //printf("File Not Found");
        }
    }
}



//判断当前程序是否以管理员权限运行
bool IsAdmin()
{
    BOOL b;
    SID_IDENTIFIER_AUTHORITY NtAuthority = SECURITY_NT_AUTHORITY;
    PSID AdministratorsGroup;
    b = AllocateAndInitializeSid(&NtAuthority, 2, SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_ADMINS, 0, 0, 0, 0, 0, 0, &AdministratorsGroup);
    if (b)
    {
        if (!CheckTokenMembership(NULL, AdministratorsGroup, &b))
        {
            b = FALSE;
        }
        FreeSid(AdministratorsGroup);
    }

    return(b);
}



bool IsSettingAdmin()
{
   
    int value = GetPrivateProfileInt(SECTION_NAME,KEY_NAME, 0, SETTING_PATH);
    

    if (value != 1) return false;
    else return true;
   

}

//获取exe所在目录 
string GetExePathEXFileName()
{
    char cModulePath[MAX_PATH] = { 0 };
    GetModuleFileNameA(NULL, cModulePath, MAX_PATH);
    //printf("cModulePath:%s\n", cModulePath);
    PathRemoveFileSpecA(cModulePath);
    string strPath = cModulePath;
    //std::wcout.imbue(std::locale("chs"));
    //wcout << strPath << "\n" << endl;//wcout 不能直接显示中文
    return strPath;
}





