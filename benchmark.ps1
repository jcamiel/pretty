$iter = [int]$args[0]


Write-Host "Running pretty (iter=$iter):"
Measure-Command { .\target\release\pretty.exe --iter $iter 5mb.json | Out-Null }

Write-Host ""
Write-Host "Running pretty --no-color (iter=$iter):"
Measure-Command { .\target\release\pretty.exe --iter $iter --no-color 5mb.json | Out-Null }

Write-Host ""
Write-Host "Running pretty --serde (iter=$iter):"
Measure-Command { .\target\release\pretty.exe --iter $iter --serde 5mb.json | Out-Null }
