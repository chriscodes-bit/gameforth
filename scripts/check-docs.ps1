$ErrorActionPreference = 'Stop'

function AnchorFromHeading($line) {
    if ($line -match '^#{1,6}\s+(.*)$') {
        $t = $Matches[1].Trim()
        # GitHub-style anchor: lowercase, strip punctuation, spaces to dashes
        $a = $t.ToLower()
        $a = ($a -replace '[^\w\- ]', '')
        $a = ($a -replace ' ', '-')
        return $a
    }
    return $null
}

$root = Join-Path $PSScriptRoot '..'
$files = @()

Get-ChildItem -Path $root -Filter '*.md' | ForEach-Object { $files += $_.FullName }
Get-ChildItem -Path (Join-Path $root 'docs') -Filter '*.md' -Recurse | ForEach-Object { $files += $_.FullName }

$problems = @()
$allAnchors = @{}

foreach ($f in $files) {
    $lines = Get-Content -Path $f
    $anchors = @()
    foreach ($l in $lines) {
        $a = AnchorFromHeading $l
        if ($a) { $anchors += $a }
    }
    $allAnchors[[System.IO.Path]::GetFullPath($f)] = $anchors

    # fence balance
    $fenceCount = 0
    foreach ($l in $lines) {
        if ($l -match '^\s*```') { $fenceCount++ }
    }
    if ($fenceCount % 2 -ne 0) {
        $problems += "UNBALANCED FENCES: $f ($fenceCount)"
    }

    # links
    $i = 0
    foreach ($l in $lines) {
        $i++
        foreach ($m in [regex]::Matches($l, '\[[^\]]*\]\(([^)]+)\)')) {
            $target = $m.Groups[1].Value
            if ($target -match '^http') { continue }
            $anchor = $null
            if ($target -match '#([^#]+)$') {
                $anchor = $Matches[1]
                if ($target -match '^([^#]+)#') { $target = $Matches[1] } else { $target = '' }
            }
            if ($target -match '^\{\{') { continue }
            if ($target -eq '') {
                $resolved = [System.IO.Path]::GetFullPath($f)
            } elseif ([System.IO.Path]::IsPathRooted($target)) {
                $resolved = [System.IO.Path]::GetFullPath($target)
            } else {
                $resolved = [System.IO.Path]::GetFullPath((Join-Path (Split-Path $f) $target))
            }
            if (-not (Test-Path $resolved)) {
                $problems += "MISSING FILE: $f`:$i -> $target"
                continue
            }
            if ($anchor) {
                $key = [System.IO.Path]::GetFullPath($resolved)
                if (-not $allAnchors.ContainsKey($key) -and (Test-Path $resolved -PathType Leaf)) {
                    # load anchors lazily
                    $al = @()
                    foreach ($l2 in (Get-Content -Path $resolved)) { $a2 = AnchorFromHeading $l2; if ($a2) { $al += $a2 } }
                    $allAnchors[$key] = $al
                }
                if ($allAnchors.ContainsKey($key) -and -not ($allAnchors[$key] -contains $anchor)) {
                    $problems += "MISSING ANCHOR: $f`:$i -> $target"
                }
            }
        }
    }
}

if ($problems.Count -eq 0) {
    Write-Output 'All docs OK: fences balanced, links and anchors resolve.'
} else {
    $problems
}
