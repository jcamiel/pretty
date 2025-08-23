$iter = [int]$args[0]
$file = $args[1]

Write-Host "Benchmark iter=$iter file=$file"

Write-Host ""
Write-Host "Running pretty:"
Measure-Command { .\target\release\pretty.exe --iter $iter $file | Out-Null }

Write-Host ""
Write-Host "Running pretty --no-color:"
Measure-Command { .\target\release\pretty.exe --iter $iter --no-color $file | Out-Null }

Write-Host ""
Write-Host "Running pretty --serde:"
Measure-Command { .\target\release\pretty.exe --iter $iter --serde $file | Out-Null }
