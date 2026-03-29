---@diagnostic disable: undefined-global
local vim = vim

local vue_filetypes = { "typescript", "vue" }
local vue_plugin = {
	name = "@vue/typescript-plugin",
	location = "node_modules/.bin/vue-language-server",
	languages = { "vue" },
	configNamespace = "typescript",
}

-- enable language server
vim.lsp.config("ts_ls", {
	cmd = { "node_modules/.bin/typescript-language-server", "--stdio" },
	init_options = {
		plugins = { vue_plugin },
	},
	filetypes = vue_filetypes,
})
vim.lsp.enable("ts_ls")

-- enable linter
local null_ls = require("null-ls")
local helpers = require("null-ls.helpers")
local methods = require("null-ls.methods")
null_ls.register(helpers.make_builtin({
	name = "eslint",
	method = methods.internal.FORMATTING,
	filetypes = vue_filetypes,
	factory = helpers.generator_factory,
	generator_opts = {
		command = "node_modules/.bin/eslint",
		args = { "--fix-dry-run", "--format", "json", "--stdin", "--stdin-filename", "$FILENAME" },
		to_stdin = true,
		format = "json",
		on_output = function(params)
			local parsed = params.output[1]
			return parsed
				and parsed.output
				and {
					{
						row = 1,
						col = 1,
						end_row = #vim.split(parsed.output, "\n") + 1,
						end_col = 1,
						text = parsed.output,
					},
				}
		end,
		check_exit_code = { 0, 1 },
	},
}))
