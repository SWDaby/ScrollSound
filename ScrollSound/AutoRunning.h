#pragma once
#include "framework.h"
#include "resource.h"

class AutoRunning
{
public:
	AutoRunning();
	~AutoRunning();

private:
	HKEY hKey;
	TCHAR strExeFullDir[MAX_PATH];
public:
	void AutoStart();
	void CanclePowerOn();
	bool IsAutoRunning();


};


