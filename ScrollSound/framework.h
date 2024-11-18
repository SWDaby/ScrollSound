// header.h: 标准系统包含文件的包含文件，
// 或特定于项目的包含文件
//

#pragma once

#include "targetver.h"
#define WIN32_LEAN_AND_MEAN             // 从 Windows 头文件中排除极少使用的内容
// Windows 头文件
#include <windows.h>
// C 运行时头文件
#include <stdlib.h>
#include <malloc.h>
#include <memory.h>
#include <tchar.h>

#include <iostream>

#include <ShlObj.h>
#include <shellapi.h>
#include <vector>
#include <Shlwapi.h>

using namespace std;


#define APP_NAME L"ScrollSound V1.24"
#define SECTION_NAME L"setting"
#define KEY_NAME  L"Administrator"
#define SETTING_PATH  L".\\setting.ini"