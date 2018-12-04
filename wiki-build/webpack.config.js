const TerserPlugin = require('terser-webpack-plugin')
const MiniCssExtractPlugin = require("extract-css-chunks-webpack-plugin")
const SpritePlugin = require(`svg-sprite-loader/plugin`)
const ManifestPlugin = require('webpack-manifest-plugin')
const WriteFilePlugin = require('write-file-webpack-plugin')
const HtmlWebpackPlugin = require('html-webpack-plugin')
const ScriptExtHtmlWebpackPlugin = require('script-ext-html-webpack-plugin')

const DEVELOPMENT = process.env.NODE_ENV !== 'production'

const stats = module.exports.stats = {
	warningsFilter: /export .* was not found in/,
	children: false,
	chunks: false,
	chunkModules: false,
	modules: false,
	reasons: false,
	usedExports: false,
}

module.exports.HtmlWebpackPlugin = HtmlWebpackPlugin
module.exports.ScriptExtHtmlWebpackPlugin = ScriptExtHtmlWebpackPlugin

module.exports.buildConfig = function buildConfig(opts) {
	return {
		devtool: 'source-map',
		mode: DEVELOPMENT ? 'development' : 'production',
		stats,
		node: {
			path: true
		},
		resolve: {
			extensions: [".ts", ".tsx", ".js"]
		},
		module: {
			rules: [{
					test: /\.(tsx?|jsx?)$/,
					exclude: /(node_modules|bower_components)/,
					use: [{
							loader: 'linaria/loader',
							options: {
								sourceMap: true,
								displayName: true
							}
						},
						{
							loader: 'babel-loader',
						}, {
							loader: 'ts-loader',
							options: {
								transpileOnly: true,
								compilerOptions: {
									composite: false,
									declaration: false,
									declarationMap: false
								}
							}
						}
					]
				},
				{
					test: /\.(png|jpg|gif)$/,
					use: [{
						loader: 'file-loader',
					}]
				},
				{
					test: /\.svg$/,
					loader: 'file-loader',
					include: /tsd.svg$/,
				},
				{
					test: /\.svg$/,
					loader: 'svg-sprite-loader',
					exclude: /tsd.svg$/
				},
				{
					test: /\.(ico)$/,
					use: [{
						loader: 'file-loader',
					}]
				},
				{
					test: /\.css$/,
					use: [MiniCssExtractPlugin.loader, 'css-loader']
				}
			]
		},
		optimization: {
			minimizer: [new TerserPlugin()]
		},
		plugins: [
			// new webpack.NamedModulesPlugin(),
			new MiniCssExtractPlugin({
				hot: DEVELOPMENT,
				filename: DEVELOPMENT ? "[name].css" : "[name].[contenthash].css"
			}),
			new SpritePlugin(),
			new WriteFilePlugin(),
			new ManifestPlugin()
		]
	}
}
