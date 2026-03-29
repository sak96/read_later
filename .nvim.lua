local tsserver_filetypes = { "typescript", "vue" }
local vue_plugin = {
	name = "@vue/typescript-plugin",
	location = "node_modules/.bin/vue-language-server",
	languages = { "vue" },
	configNamespace = "typescript",
}
vim.lsp.config("ts_ls", {
	cmd = { "node_modules/.bin/typescript-language-server", "--stdio" },
	init_options = {
		plugins = { vue_plugin },
	},
	filetypes = { "typescript", "vue" },
})
vim.lsp.enable("ts_ls")
