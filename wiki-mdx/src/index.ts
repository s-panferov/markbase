const mdx = require('@mdx-js/mdx')
declare const MARKDOWN: string

module.exports = function(str: string) {
	return mdx.default
		.sync(str, {
			mdPlugins: [],
			hastPlugins: [],
			skipExport: true
		})
		.trim()
}
