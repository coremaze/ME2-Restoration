#include <winsock2.h>
#include <windows.h>
#include <ws2tcpip.h>
#include <iostream>
#include <stdio.h> // Needed for freopen>

// Function pointer for the original WSAConnect
typedef int (WINAPI *WSAConnect_t)(
    SOCKET s,
    const struct sockaddr* name,
    int namelen,
    LPWSABUF lpCallerData,
    LPWSABUF lpCalleeData,
    LPQOS lpSQOS,
    LPQOS lpGQOS
);

// Function pointer for the original connect
typedef int (WINAPI *connect_t)(
    SOCKET s,
    const struct sockaddr* name,
    int namelen
);

WSAConnect_t OriginalWSAConnect = nullptr;
connect_t OriginalConnect = nullptr;

// Arrays to store original bytes
unsigned char OriginalWSAConnectBytes[5] = {0};
unsigned char OriginalConnectBytes[5] = {0};

// Function to install a jump hook
bool InstallHook(void* originalFunction, void* hookFunction, unsigned char* originalBytes) {
    // Save the original bytes
    memcpy(originalBytes, originalFunction, 5);

    DWORD oldProtect;
    if (!VirtualProtect(originalFunction, 5, PAGE_EXECUTE_READWRITE, &oldProtect)) {
        std::cerr << "Failed to change memory protection. Error: " << GetLastError() << std::endl;
        return false;
    }

    uintptr_t relativeAddress = ((uintptr_t)hookFunction - (uintptr_t)originalFunction - 5);
    unsigned char jump[5] = { 0xE9 };
    memcpy(jump + 1, &relativeAddress, sizeof(relativeAddress));
    memcpy(originalFunction, jump, 5);

    // Restore the original protection
    VirtualProtect(originalFunction, 5, oldProtect, &oldProtect);
    return true;
}

// Function to remove a jump hook
bool RemoveHook(void* originalFunction, unsigned char* originalBytes) {
    DWORD oldProtect;
    if (!VirtualProtect(originalFunction, 5, PAGE_EXECUTE_READWRITE, &oldProtect)) {
        std::cerr << "Failed to change memory protection. Error: " << GetLastError() << std::endl;
        return false;
    }

    memcpy(originalFunction, originalBytes, 5);

    // Restore the original protection
    VirtualProtect(originalFunction, 5, oldProtect, &oldProtect);
    return true;
}

// Hooked WSAConnect function
int WINAPI HookedWSAConnect(
    SOCKET s,
    const struct sockaddr* name,
    int namelen,
    LPWSABUF lpCallerData,
    LPWSABUF lpCalleeData,
    LPQOS lpSQOS,
    LPQOS lpGQOS
) {
    // std::cout << "[Hook] HookedWSAConnect called" << std::endl;
    if (name->sa_family == AF_INET) {
        sockaddr_in* addr_in = (sockaddr_in*)name;
        char ip[INET_ADDRSTRLEN];
        inet_ntop(AF_INET, &(addr_in->sin_addr), ip, INET_ADDRSTRLEN);
        int port = ntohs(addr_in->sin_port);
        std::cout << "[Hook] Attempting to connect to IP: " << ip << " Port: " << port << std::endl;

        // Example: Alter the IP address or port if needed
        // Uncomment and modify the following lines to change the destination
        /*
        inet_pton(AF_INET, "127.0.0.1", &(addr_in->sin_addr));
        addr_in->sin_port = htons(8080);
        */
    }

    // Temporarily remove the hook
    RemoveHook((void*)OriginalWSAConnect, OriginalWSAConnectBytes);

    // Call the original WSAConnect function
    int result = OriginalWSAConnect(s, name, namelen, lpCallerData, lpCalleeData, lpSQOS, lpGQOS);

    // Reinstall the hook
    if (!InstallHook((void*)OriginalWSAConnect, (void*)HookedWSAConnect, OriginalWSAConnectBytes)) {
        std::cerr << "[Hook] Failed to reinstall hook for WSAConnect." << std::endl;
    }

    return result;
}

// Hooked connect function
int WINAPI HookedConnect(
    SOCKET s,
    const struct sockaddr* name,
    int namelen
) {
    // std::cout << "[Hook] HookedConnect called" << std::endl;
    if (name->sa_family == AF_INET) {
        sockaddr_in* addr_in = (sockaddr_in*)name;
        char ip[INET_ADDRSTRLEN];
        inet_ntop(AF_INET, &(addr_in->sin_addr), ip, INET_ADDRSTRLEN);
        int port = ntohs(addr_in->sin_port);
        std::cout << "[Hook] Attempting to connect to IP: " << ip << " Port: " << port << std::endl;

        // Example: Alter the IP address or port if needed
        // Uncomment and modify the following lines to change the destination
        /*
        inet_pton(AF_INET, "127.0.0.1", &(addr_in->sin_addr));
        addr_in->sin_port = htons(8080);
        */

        // inet_pton(AF_INET, "10.0.2.2", &(addr_in->sin_addr));
        inet_pton(AF_INET, "127.0.0.1", &(addr_in->sin_addr));

    }

    {
        sockaddr_in* addr_in = (sockaddr_in*)name;
        char ip[INET_ADDRSTRLEN];
        inet_ntop(AF_INET, &(addr_in->sin_addr), ip, INET_ADDRSTRLEN);
        std::cout << "[Hook] Redirecting connection to " << ip << std::endl;
    }

    // Temporarily remove the hook
    RemoveHook((void*)OriginalConnect, OriginalConnectBytes);

    // Call the original connect function
    int result = OriginalConnect(s, name, namelen);

    // Reinstall the hook
    if (!InstallHook((void*)OriginalConnect, (void*)HookedConnect, OriginalConnectBytes)) {
        std::cerr << "[Hook] Failed to reinstall hook for connect." << std::endl;
    }

    return result;
}

// Function to set up the hooks
bool SetupHook() {
    HMODULE hWs2_32 = GetModuleHandleA("ws2_32.dll");
    if (!hWs2_32) {
        std::cerr << "Failed to get handle of ws2_32.dll. Error: " << GetLastError() << std::endl;
        return false;
    }

    OriginalWSAConnect = (WSAConnect_t)GetProcAddress(hWs2_32, "WSAConnect");
    if (!OriginalWSAConnect) {
        std::cerr << "Failed to get address of WSAConnect. Error: " << GetLastError() << std::endl;
        return false;
    }

    OriginalConnect = (connect_t)GetProcAddress(hWs2_32, "connect");
    if (!OriginalConnect) {
        std::cerr << "Failed to get address of connect. Error: " << GetLastError() << std::endl;
        return false;
    }

    // Install hooks
    if (!InstallHook((void*)OriginalWSAConnect, (void*)HookedWSAConnect, OriginalWSAConnectBytes)) {
        return false;
    }

    if (!InstallHook((void*)OriginalConnect, (void*)HookedConnect, OriginalConnectBytes)) {
        return false;
    }

    return true;
}

BOOL APIENTRY DllMain(HMODULE hModule,
                      DWORD  ul_reason_for_call,
                      LPVOID lpReserved
) {
    switch (ul_reason_for_call)
    {
    case DLL_PROCESS_ATTACH:
        DisableThreadLibraryCalls(hModule);

        // Allocate a new console for the DLL
        if (AllocConsole()) {
            freopen("CONOUT$", "w", stdout);
            freopen("CONOUT$", "w", stderr);
            std::cout << "[Hook] Console allocated successfully." << std::endl;
        } else {
            std::cerr << "[Hook] Failed to allocate console." << std::endl;
        }

        std::cout << "[Hook] DLL_PROCESS_ATTACH" << std::endl;
        if (SetupHook()) {
            std::cout << "[Hook] Hook set up successfully." << std::endl;
        } else {
            std::cerr << "[Hook] Failed to set up hook." << std::endl;
        }
        break;
    case DLL_PROCESS_DETACH:
        // Optionally, free the console when the DLL is detached
        FreeConsole();
        break;
    }
    return TRUE;
}
