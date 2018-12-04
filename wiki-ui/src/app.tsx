import React from 'react'
import { css } from 'linaria'
const { hot } = require('react-hot-loader')

export class App extends React.Component {
	render() {
		return <h1>hello</h1>
	}
}

export default hot(module)(App)
