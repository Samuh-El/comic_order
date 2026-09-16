# generate-skills.ps1
$PSScriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Definition
. "$PSScriptRoot/common.ps1"
$repoRoot = Get-RepoRoot
$agentsDir = Join-Path $repoRoot ".github/agents"
$skillsDir = Join-Path $repoRoot ".agents/skills"

# Crear directorio de habilidades si no existe
if (-not (Test-Path $skillsDir)) {
    New-Item -ItemType Directory -Path $skillsDir -Force | Out-Null
}

# Leer todos los agentes speckit
$agentFiles = Get-ChildItem -Path $agentsDir -Filter "speckit.*.agent.md"
foreach ($file in $agentFiles) {
    if ($file.Name -match '^speckit\.([^\.]+)\.agent\.md$') {
        $cmdName = $Matches[1]
        $content = Get-Content -Path $file.FullName -Raw
        
        # Separar YAML Frontmatter del cuerpo de Markdown
        $lines = $content -split "`r?`n"
        $firstDash = -1; $secondDash = -1
        for ($i = 0; $i -lt $lines.Count; $i++) {
            if ($lines[$i] -eq '---') {
                if ($firstDash -eq -1) { $firstDash = $i } else { $secondDash = $i; break }
            }
        }
        
        $frontmatterContent = ""; $bodyContent = ""
        if ($firstDash -ne -1 -and $secondDash -ne -1) {
            $frontmatterContent = ($lines[($firstDash+1)..($secondDash-1)]) -join "`n"
            $bodyContent = ($lines[($secondDash+1)..($lines.Count-1)]) -join "`n"
        } else {
            $bodyContent = $content
        }
        
        # Extraer descripción existente
        $description = ""
        if ($frontmatterContent -match 'description:\s*(.*)') {
            $description = $Matches[1].Trim().Trim('"').Trim("'")
        }
        
        # 1. Crear variante con separador de barra inclinada (speckit/nombre)
        $skillNameSlash = "speckit/$cmdName"
        $skillFolderSlash = Join-Path $skillsDir "speckit-$cmdName-slash"
        New-Item -ItemType Directory -Path $skillFolderSlash -Force | Out-Null
        
        $newFrontmatterSlash = @"
---
name: $skillNameSlash
$frontmatterContent
---
"@
        [System.IO.File]::WriteAllText((Join-Path $skillFolderSlash "SKILL.md"), "$newFrontmatterSlash`n$bodyContent", [System.Text.Encoding]::UTF8)
        
        # 2. Crear variante con separador de punto (speckit.nombre)
        $skillNameDot = "speckit.$cmdName"
        $skillFolderDot = Join-Path $skillsDir "speckit-$cmdName-dot"
        New-Item -ItemType Directory -Path $skillFolderDot -Force | Out-Null
        
        $newFrontmatterDot = @"
---
name: $skillNameDot
$frontmatterContent
---
"@
        [System.IO.File]::WriteAllText((Join-Path $skillFolderDot "SKILL.md"), "$newFrontmatterDot`n$bodyContent", [System.Text.Encoding]::UTF8)
    }
}
Write-Output "Successfully regenerated skills in .agents/skills/"
