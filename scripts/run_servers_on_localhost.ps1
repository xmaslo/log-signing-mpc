# Get current working directory
$currentDirectory = $PWD.Path

# Get the relative path from the command-line argument
$relativePath = $args[0]

# Get the absolute path by combining the current directory with the relative path
$absolutePath = Join-Path -Path $PWD -ChildPath $relativePath

# Check if the directory exists
if (Test-Path -Path $absolutePath -PathType Container) {
    # Change the working directory to the absolute path
    Set-Location -Path $absolutePath

    # Additional command line arguments for the program
    $sArguments1 = "1", "8000", "3000"
    $sArguments2 = "2", "8001", "3001"
    $sArguments3 = "3", "8002", "3002"

    # 1 server
    Write-Host "Running program: "log-signing-mpc.exe" $sArguments1"
    Start-Process -FilePath "log-signing-mpc.exe" -ArgumentList $sArguments1

    # 2 server
    Write-Host "Running program: "log-signing-mpc.exe" $sArguments2"
    Start-Process -FilePath "log-signing-mpc.exe" -ArgumentList $sArguments2

    # 3 server
    Write-Host "Running program: "log-signing-mpc.exe" $sArguments3"
    Start-Process -FilePath "log-signing-mpc.exe" -ArgumentList $sArguments3

    # return current working directory back to the previous version
    Set-Location -Path $currentDirectory
}
else {
    Write-Host "Directory not found: $absolutePath"
}
