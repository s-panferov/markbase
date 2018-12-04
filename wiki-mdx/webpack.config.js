const {
	buildConfig,
	stats
} = require('../wiki-build/webpack.config')

let config = buildConfig({
	rootPath: __dirname
})

config.entry = {
	index: './src/index.ts'
}

config.output = {
	library: 'exports',
	libraryTarget: 'assign'
}

config.externals = {
	'react-dom/server': 'ReactDOM.Server',
	'react-dom': 'ReactDOM',
	react: 'React'
}

module.exports = config
