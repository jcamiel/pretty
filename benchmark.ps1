

Write-Host "Running pretty (iter=500):"
Measure-Command { .\target\release\pretty.exe --iter 500 < 5mb.json | Out-Null }

Write-Host ""
Write-Host "Running pretty --no-color (iter=500):"
Measure-Command { .\target\release\pretty.exe --iter 500 --no-color < 5mb.json | Out-Null }

Write-Host ""
Write-Host "Running pretty pretty --serde (iter=500):"
Measure-Command { .\target\release\pretty.exe --iter 500 --serde < 5mb.json | Out-Null }
