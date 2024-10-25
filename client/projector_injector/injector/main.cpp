#include <windows.h>
#include <string>
#include <iostream>

bool InjectDLL(const std::string& processPath, const std::string& dllPath, const std::string& arguments) {
    STARTUPINFOA si = { sizeof(si) };
    PROCESS_INFORMATION pi;

    // Prepare the command line with arguments
    std::string commandLine = "\"" + processPath + "\" " + arguments;

    // Create the process in a suspended state
    if (!CreateProcessA(
            NULL,
            const_cast<char*>(commandLine.c_str()),
            NULL,
            NULL,
            FALSE,
            CREATE_SUSPENDED,
            NULL,
            NULL,
            &si,
            &pi)) {
        std::cerr << "Failed to create process. Error: " << GetLastError() << std::endl;
        return false;
    }

    // Allocate memory in the target process for the DLL path
    void* allocMem = VirtualAllocEx(pi.hProcess, NULL, dllPath.length() + 1, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
    if (!allocMem) {
        std::cerr << "Failed to allocate memory in target process. Error: " << GetLastError() << std::endl;
        TerminateProcess(pi.hProcess, 1);
        return false;
    }

    // Write the DLL path to the allocated memory
    if (!WriteProcessMemory(pi.hProcess, allocMem, dllPath.c_str(), dllPath.length() + 1, NULL)) {
        std::cerr << "Failed to write to process memory. Error: " << GetLastError() << std::endl;
        VirtualFreeEx(pi.hProcess, allocMem, 0, MEM_RELEASE);
        TerminateProcess(pi.hProcess, 1);
        return false;
    }

    // Get the address of LoadLibraryA
    LPVOID loadLibraryAddr = (LPVOID)GetProcAddress(GetModuleHandleA("kernel32.dll"), "LoadLibraryA");
    if (!loadLibraryAddr) {
        std::cerr << "Failed to get LoadLibraryA address. Error: " << GetLastError() << std::endl;
        VirtualFreeEx(pi.hProcess, allocMem, 0, MEM_RELEASE);
        TerminateProcess(pi.hProcess, 1);
        return false;
    }

    // Create a remote thread that calls LoadLibraryA with the DLL path
    HANDLE remoteThread = CreateRemoteThread(pi.hProcess, NULL, 0, 
        (LPTHREAD_START_ROUTINE)loadLibraryAddr, allocMem, 0, NULL);
    if (!remoteThread) {
        std::cerr << "Failed to create remote thread. Error: " << GetLastError() << std::endl;
        VirtualFreeEx(pi.hProcess, allocMem, 0, MEM_RELEASE);
        TerminateProcess(pi.hProcess, 1);
        return false;
    }

    // Wait for the remote thread to finish
    WaitForSingleObject(remoteThread, INFINITE);

    // Clean up
    VirtualFreeEx(pi.hProcess, allocMem, 0, MEM_RELEASE);
    CloseHandle(remoteThread);

    // Resume the main thread of the process
    if (ResumeThread(pi.hThread) == -1) {
        std::cerr << "Failed to resume thread. Error: " << GetLastError() << std::endl;
        TerminateProcess(pi.hProcess, 1);
        return false;
    }

    // Close handles
    CloseHandle(pi.hThread);
    CloseHandle(pi.hProcess);

    std::cout << "DLL injected successfully." << std::endl;
    return true;
}

int main(int argc, char* argv[]) {
    if (argc < 3) {
        std::cerr << "Usage: Injector.exe <ProcessPath> <DLLPath> [Arguments]" << std::endl;
        return 1;
    }

    std::string processPath = argv[1];
    std::string dllPath = argv[2];
    std::string arguments;

    if (argc > 3) {
        for (int i = 3; i < argc; ++i) {
            arguments += std::string(argv[i]) + " ";
        }
    }

    if (!InjectDLL(processPath, dllPath, arguments)) {
        return 1;
    }

    return 0;
}