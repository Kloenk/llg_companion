module.exports = {
	"parser": "babel-eslint",
	"extends": "eslint-config-standard",
	"rules": {
		"indent": ["error", "tab"],
		"no-tabs": "off",
		"semi": [2, "always"],
		"no-extra-semi": 2,
		"semi-spacing": [2, { "before": false, "after": true }],
		"no-template-curly-in-string": "off"
	},
	"env": {"node": true, "browser": true, "worker": true}
};
