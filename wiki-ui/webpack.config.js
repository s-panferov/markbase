const {
	buildConfig,
	HtmlWebpackPlugin,
	ScriptExtHtmlWebpackPlugin,
	stats
} = require('../wiki-build/webpack.config')

let config = buildConfig({
	rootPath: __dirname
})

config.entry = {
	index: './src/index.tsx'
}

config.output = {
	publicPath: "/",
}

config.externals = {
	'react-dom/server': 'ReactDOM.Server',
	'react-dom': 'ReactDOM',
	react: 'React'
}

config.devServer = {
	stats,
	contentBase: 'dist',
	historyApiFallback: true,
	hot: true,
	port: 9000
}

config.plugins.push(new HtmlWebpackPlugin({
	inject: 'head',
	template: path.join(opts.rootPath, 'index.html')
}))

config.plugins.push(new ScriptExtHtmlWebpackPlugin({
	defaultAttribute: 'defer'
}))

module.exports = config
