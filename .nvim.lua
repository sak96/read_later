vim.lsp.config('rust_analyzer',{
  settings = {
    ['rust-analyzer'] = {
      -- Optional: Disable cache priming to save startup RAM
      cachePriming = {
        enable = false,
      },
      checkOnSave = {
        enable = true, -- Keep enabled, but be aware it triggers cargo check
      },
      procMacro = {
        enable = true,  -- REQUIRED for Yew
      },
      cargo = {
        -- Prevents analyzing all targets if you only care about the one you're editing
        targetDir = "target/analyzer",
        allTargets = false,
      }
    },
  },
})
