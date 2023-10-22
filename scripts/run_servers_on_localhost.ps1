# Get current working directory
$currentDirectory = $PWD.Path

# Check if any command-line argument was provided
if ($args.Length -ne 3) {
    Write-Host "Usage: .\run_servers_on_localhost.ps1 path\to\the\app\binary threshold numServers"
    return
}

# Get the relative path from the command-line argument
$relativePath = $args[0]

# Get the absolute path by combining the current directory with the relative path
$absolutePath = Join-Path -Path $PWD -ChildPath $relativePath

# Check if the directory exists
if (Test-Path -Path $absolutePath -PathType Container) {
    # Change the working directory to the absolute path
    Set-Location -Path $absolutePath

    $threshold = [int]$args[1]
    $numServers = [int]$args[2]
    # Define the arguments for the program
    $programArguments = @()

    for ($i = 1; $i -le $numServers; $i++) {
        $programArguments += "$i", "800$i", "300$i", "$threshold", "$numServers"
    }

    # Start the "log-signing-mpc.exe" processes
    for ($i = 1; $i -le $numServers; $i++) {
        $exePath = "log-signing-mpc.exe"
        Write-Host "Running program: $exePath $($programArguments[(($i - 1) * 5)..($i * 5 - 1)])"
        Start-Process -FilePath $exePath -ArgumentList $programArguments[(($i - 1) * 5)..($i * 5 - 1)]
    }

    # Return the current working directory back to the previous version
    Set-Location -Path $currentDirectory
}
else {
    Write-Host "Directory not found: $absolutePath"
}