'use strict';

import { render, html } from 'lit-html';

/* fck jqry */

const id = document.getElementById.bind(document);

/* definitions */
// allow for better uglifying

const body = document.body;
const localStorage = window.localStorage;
const sw = navigator.serviceWorker;
const search = id('search');

/* variables */

let weekShift = 0;
let data = {};

/* actions */

search.value = localStorage.getItem('selected');

if (search.value) {
	fetchPlan();
} else {
	search.focus();
}

if (!navigator.onLine) {
	body.classList.add('o');
	body.classList.remove('l');
}

/* events */

addEvent(id('clear'), 'click', function clear () {
	search.value = '';
	search.focus();
});

addEvent(id('searchicon'), 'click', function clear () {
	search.focus();
});

addEvent(search, 'blur', function () {
	body.classList.remove('typing');
});

addEvent(window, 'online', function () {
	body.classList.remove('o');
	checkAuth();
});

checkAuth();

addEvent(window, 'offline', function () {
	body.classList.add('o');
});

addEvent(window, 'focus', function () {
	if (search.value) {
		let value = encodeURIComponent(search.value);
		fetch('v2/plan.json?name=' + value, { credentials: 'same-origin' })
			.then(showLogin);
	}
});

addEvent(id('lastweek'), 'click', function () {
	weekShift++;
	renderPage();
});

addEvent(id('nextweek'), 'click', function () {
	weekShift++;
	renderPage();
});

function getActiveDate () {
	let date = new Date();
	date.setHours(date.getHours() + 8);
	while (date.getDay() === 6 || date.getDay() === 0) date.setDate(date.getDate() + 1);
	return date;
}

function getWeekNum (activeDate) {
	let date = new Date(activeDate);
	let onejan = new Date(date.getFullYear(), 0, 1);
	return Math.ceil(((date - onejan) / 86400000 + onejan.getDay() + 1) / 7) % 2;
}

function getActiveWeek (activeDate) {
	let date = new Date(activeDate);
	date.setDate(date.getDate() + weekShift * 7);
	return date;
}

function save () {
	body.classList.remove('nd');
	id('ac-ss').innerHTML = '';
	if (sw && sw.controller) {
		sw.controller.postMessage(
			JSON.stringify({
				type: 'prerender',
				content: document.documentElement.outerHTML
			})
		);
	}
}

function renderPage () {
	checkAuth();
	let date = getActiveDate();
	let activeWeek = getWeekNum(getActiveWeek(date));
	let currentWeek = getWeekNum(date);
	render(html`
		<div id="spacer"></div>
		${data.t.map((week, wid) => {
			let weekLetter = String.fromCharCode('A'.charCodeAt(0) + wid);
			return html`
				<div class="p ${activeWeek === wid ? 'tw' : ''}" id="p${wid}" class="p">
					<h4 id="title-${wid}">Woche ${weekLetter}</h4>
					<table class="centered striped card">
						<tbody id="table-${wid}">
							<tr>
								<th class="hour"></th>
								${['Mo', 'Di', 'Mi', 'Do', 'Fr'].map((day, did) => {
									return html`
										<th class="${(currentWeek === wid && date.getDay() -1 === did) ? 'today ' : ''}">${day}</th>
									`;
								})}
							</tr>
							${week.map((hour, hid) => {
								return html`
									<tr>
										<td class="hour">${hid + 1}</td>
										${hour.map((day, did) => {
											let text = day, classes = (currentWeek === wid && date.getDay() -1 === did) ? 'today ' : '';
											if (Array.isArray(day)) {
												text = day[0];
												classes += day[1];
											}
											text = text.split(' ');
											return html`
												<td class="${classes}"><p><b>${text[0]}</b></p>${text.slice(1).map(t => html`<p>${t}</p>`)}</td>
											`;
										})}
									</tr>
								`;
							})}
						</tbody>
					</table>
				</div>
			`;
		})}
		<footer>
			<span>
				Version <a href="__GIT_URL" target="_blank">__GIT_REVISION</a> | i
				Stundenplan zuletzt aktualisiert am ${formatDate(new Date(data.d))} | 
				<a href="__IMPRESSUM_URL">Impressum</a> | 
				<a href="/login.html" id="nd-l">login</a> <em id="nd-l">|</em>
				<a id="js-toggle-color" href="#">Toggle theme</a>
			</span>
		</footer>
	`, id('content'));
	id('js-toggle-color').onclick = function (e) {
		let classes = document.body.classList;
		if (classes.contains('dark')) {
			classes.remove('dark');
		} else {
			classes.add('dark');
		}
	};

	search.blur();
	save();
}

function pad (string, amount) {
	return (
		Array(amount)
			.join('0')
			.substr(0, amount - String(string).length) + string
	);
}

function formatDate (date) {
	return (
		'' +
		pad(date.getDate(), 2) +
		'.' +
		pad(date.getMonth() + 1, 2) +
		'.' +
		date.getFullYear() +
		' um ' +
		date.getHours() +
		':' +
		pad(date.getMinutes(), 2) +
		' Uhr'
	);
}

function addEvent (el, type, handler) {
	el.addEventListener(type, handler);
}

function source (val, suggest) {
	let value = encodeURIComponent(val);
	fetch('names.json?name=' + value, { credentials: 'same-origin' }).then(showLogin)
		.then(function (resp) {
			return resp.json();
		})
		.then(function (data) {
			suggest(data.names);
		});
}

function renderItem (item, search) {
	// escape special characters
	search = search.replace(/[-/\\^$*+?.()|[\]{}]/g, '\\$&');
	let re = new RegExp('(' + search.split(' ').join('|') + ')', 'gi');
	return (
		'<div class="ac-s" data-val="' +
		item +
		'">' +
		item.replace(re, '<span class="hl">$1</span>') +
		'</div>'
	);
}

function onSelect (e, term, item) {
	body.classList.remove('typing');
	search.value = term;
	localStorage.setItem('selected', search.value);
	fetchPlan();
}

function fetchPlan () {
	let value = encodeURIComponent(search.value);
	fetch('v2/plan.json?name=' + value, { credentials: 'same-origin' }).then(showLogin)
		.then(function (resp) {
			return resp.json();
		})
		.then(function (json) {
			data = json;
			renderPage();
		});
}

/*
	Based on JavaScript autoComplete v1.0.4
	Copyright (c) 2014 Simon Steinberger / Pixabay
	Copyright (c) 2017-2018 PetaByteBoy
	GitHub: https://github.com/Pixabay/JavaScript-autoComplete
	License: http://www.opensource.org/licenses/mit-license.php
*/

function addEventToSuggestions (event, cb) {
	addEvent(id('ac-ss'), event, function (e) {
		let found;
		let el = e.target || e.srcElement;
		while (el && !(found = el.classList.contains('ac-s'))) { el = el.parentElement; }
		if (found) cb.call(el, e);
	});
}

search.last_val = '';
body.appendChild(id('ac-ss'));

addEventToSuggestions('mouseleave', function (e) {
	let sel = id('ac-ss').querySelector('.ac-s.s');
	if (sel) {
		setTimeout(function () {
			sel.classList.remove('s');
		}, 20);
	}
});

addEventToSuggestions('mouseover', function (e) {
	let sel = id('ac-ss').querySelector('.ac-s.s');
	if (sel) sel.classList.remove('s');
	this.classList.add('s');
});

addEventToSuggestions('mousedown', function (e) {
	if (this.classList.contains('ac-s')) {
		// else outside click
		let v = this.getAttribute('data-val');
		search.value = v;
		onSelect(e, v, this);
	}
});

function suggest (data) {
	let val = search.value;
	if (data.length && val.length > 1) {
		let s = '';
		for (let i = 0; i < data.length; i++) s += renderItem(data[i], val);
		id('ac-ss').innerHTML = s;
		body.classList.add('typing');
	} else id('ac-ss').innerHTML = '';
}

addEvent(search, 'keydown', function (e) {
	let key = window.event ? e.keyCode : e.which;
	// down (40), up (38)
	if (key === 9) {
		e.preventDefault();
		key = 40;
	}
	if ((key === 40 || key === 38) && id('ac-ss').innerHTML) {
		let next;
		let sel = id('ac-ss').querySelector('.ac-s.s');
		if (!sel) {
			next =
				key === 40
					? id('ac-ss').querySelector('.ac-s')
					: id('ac-ss').childNodes[id('ac-ss').childNodes.length - 1]; // first : last
			next.classList.add('s');
		} else {
			next = key === 40 ? sel.nextSibling : sel.previousSibling;
			if (next) {
				sel.classList.remove('s');
				next.classList.add('s');
			} else {
				sel.classList.remove('s');
				search.value = search.last_val;
				next = 0;
			}
		}
		body.classList.add('typing');

		return false;
	} else if (key === 27) {
		// esc
		search.value = search.last_val;
		search.blur();
	} else if (key === 13 || key === 9) {
		// enter
		if (body.classList.contains('typing')) {
			let sel = id('ac-ss').querySelector('.ac-s.s') || id('ac-ss').firstChild;
			onSelect(e, sel.getAttribute('data-val'), sel);
		}
	}
});

addEvent(search, 'keyup', function (e) {
	let key = window.event ? e.keyCode : e.which;
	if (!key || ((key < 35 || key > 40) && key !== 13 && key !== 27)) {
		let val = search.value;
		if (val.length > 1) {
			if (val !== search.last_val) {
				search.last_val = val;
				clearTimeout(search.timer);
				source(val, suggest);
			}
		} else {
			search.last_val = val;
			body.classList.remove('typing');
		}
	}
});

function checkAuth () {
	fetch('check', { credentials: 'same-origin' })
		.then(showLogin);
}

// shows login button
function showLogin (resp) {
	if (resp.status === 401) {
		console.log('refreshing key');
		body.classList.add('l');
		return {};	// return empty struct for json decoding
	} else if (resp.status === 200) {
		body.classList.remove('l');
	}
	return resp;
}

// sw

if (sw) {
	sw.register('sw.js', {
		scope: './'
	});

	sw.onmessage = function (evt) {
		data = JSON.parse(evt.data);
		renderPage();
	};
}
