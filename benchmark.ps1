$iter = [int]$args[0]


Write-Host "Running pretty (iter=500):"
Measure-Command { Get-Content -Raw -Encoding UTF8 5mb.json | .\target\release\pretty.exe --iter $iter | Out-Null }

Write-Host ""
Write-Host "Running pretty --no-color (iter=500):"
Measure-Command { Get-Content -Raw -Encoding UTF8 5mb.json | .\target\release\pretty.exe --iter $iter --no-color | Out-Null }

Write-Host ""
Write-Host "Running pretty pretty --serde (iter=500):"
Measure-Command { Get-Content -Raw -Encoding UTF8 5mb.json | .\target\release\pretty.exe --iter $iter --serde | Out-Null }
