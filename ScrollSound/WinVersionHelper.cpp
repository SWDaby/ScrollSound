#include "WinVersionHelper.h"



CWinVersionHelper::CWinVersionHelper()
{
	DWORD dwMajorVer{}, dwMinorVer{}, dwBuildNumber{};
	HMODULE hModNtdll{};
	if (hModNtdll = ::LoadLibrary(_T("ntdll.dll")))
	{
		typedef void (WINAPI *pfRTLGETNTVERSIONNUMBERS)(DWORD*, DWORD*, DWORD*);
		pfRTLGETNTVERSIONNUMBERS pfRtlGetNtVersionNumbers;
		pfRtlGetNtVersionNumbers = (pfRTLGETNTVERSIONNUMBERS)::GetProcAddress(hModNtdll, "RtlGetNtVersionNumbers");
		if (pfRtlGetNtVersionNumbers)
		{
			pfRtlGetNtVersionNumbers(&dwMajorVer, &dwMinorVer, &dwBuildNumber);
			dwBuildNumber &= 0x0ffff;
		}
		::FreeLibrary(hModNtdll);
		hModNtdll = NULL;
	}
	m_major_version = dwMajorVer;
	m_minor_version = dwMinorVer;
	m_build_number = dwBuildNumber;

   
}

CWinVersionHelper::~CWinVersionHelper()
{
}

//Windows 11 的发布周期没有包含“XXH1”更新，所有主要更新都使用“H2”命名
//https://zh.wikipedia.org/wiki/Windows_11%E7%89%88%E6%9C%AC%E5%8E%86%E5%8F%B2

bool CWinVersionHelper::IsWindows11OrLater() const
{
	if (m_major_version > 10)
		return true;
	else if (m_major_version == 10 && m_minor_version > 0)
		return true;
	else if (m_major_version == 10 && m_minor_version == 0 && m_build_number >= 21996)
		return true;
	else return false;
}

bool CWinVersionHelper::IsWindows11_21H2() const
{
	
	if (m_major_version == 10 && m_minor_version == 0 && m_build_number == 22000)
		return true;
	else return false;
}

bool CWinVersionHelper::IsWindows11_22H2() const
{

	if (m_major_version == 10 && m_minor_version == 0 && m_build_number == 22621)
		return true;
	else return false;
}

bool CWinVersionHelper::IsWindows11_22H2OrLater() const
{

	if (m_major_version == 10 && m_minor_version == 0 && m_build_number >= 22621)
		return true;
	else return false;
}


//==================================== = Windows10=====================================
//https://zh.wikipedia.org/wiki/Windows_10%E7%89%88%E6%9C%AC%E5%8E%86%E5%8F%B2

bool CWinVersionHelper::IsWindows10FallCreatorOrLater() const
{
	if (m_major_version > 10)
		return true;
	else if (m_major_version == 10 && m_minor_version > 0)
		return true;
	else if (m_major_version == 10 && m_minor_version == 0 && m_build_number >= 16299)
		return true;
	else return false;
}



bool CWinVersionHelper::IsWindows10OrLater() const
{
    return m_major_version >= 10;
}


bool CWinVersionHelper::IsWindows10() const
{
	if (m_major_version == 10 && m_minor_version == 0 && m_build_number <= 21390)
		return true;
	else return false;
	
}

//==================================== = Windows7=====================================

bool CWinVersionHelper::IsWindows7() const
{
	return (m_major_version == 6 && m_minor_version == 1);
}


//==================================== = Windows8=====================================
bool CWinVersionHelper::IsWindows8Or8point1() const
{
	return (m_major_version == 6 && m_minor_version > 1);
}

bool CWinVersionHelper::IsWindows8OrLater() const
{
	if (m_major_version > 6)
		return true;
	else if (m_major_version == 6 && m_minor_version >= 2)
		return true;
	else return false;
}