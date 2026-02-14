// Common emoji shortcodes → unicode mappings
// Sorted by frequency of use in chat
export const EMOJI_MAP: Record<string, string> = {
	// Smileys
	smile: '😄', laughing: '😆', blush: '😊', heart_eyes: '😍', kissing_heart: '😘',
	wink: '😉', thinking: '🤔', neutral_face: '😐', expressionless: '😑', unamused: '😒',
	sweat: '😓', pensive: '😔', confused: '😕', upside_down: '🙃', money_mouth: '🤑',
	astonished: '😲', worried: '😟', frowning: '😦', anguished: '😧', fearful: '😨',
	weary: '😩', exploding_head: '🤯', flushed: '😳', crazy: '🤪', shushing: '🤫',
	vomiting: '🤮', cowboy: '🤠', clown: '🤡', nerd: '🤓', monocle: '🧐',
	joy: '😂', sob: '😭', cry: '😢', angry: '😠', rage: '🤬',
	rofl: '🤣', slightly_smiling: '🙂', grinning: '😀', grin: '😁', smiley: '😃',
	stuck_out_tongue: '😛', stuck_out_tongue_winking_eye: '😜', stuck_out_tongue_closed_eyes: '😝',
	drooling: '🤤', yawning: '🥱', sleeping: '😴', mask: '😷', sunglasses: '😎',
	smirk: '😏', relieved: '😌', scream: '😱', cold_sweat: '😰', hugging: '🤗',
	rolling_eyes: '🙄', shrug: '🤷', facepalm: '🤦', skull: '💀',
	// Hands
	thumbsup: '👍', thumbsdown: '👎', ok_hand: '👌', wave: '👋',
	clap: '👏', pray: '🙏', handshake: '🤝', muscle: '💪',
	point_up: '☝️', point_down: '👇', point_left: '👈', point_right: '👉',
	raised_hand: '✋', middle_finger: '🖕', v: '✌️', crossed_fingers: '🤞',
	love_you: '🤟', metal: '🤘', call_me: '🤙', palms_up: '🤲',
	fist: '✊', punch: '👊', writing_hand: '✍️', nail_care: '💅',
	// Hearts
	heart: '❤️', orange_heart: '🧡', yellow_heart: '💛', green_heart: '💚',
	blue_heart: '💙', purple_heart: '💜', black_heart: '🖤', white_heart: '🤍',
	broken_heart: '💔', sparkling_heart: '💖', heartbeat: '💓', two_hearts: '💕',
	revolving_hearts: '💞', heartpulse: '💗', gift_heart: '💝', heart_on_fire: '❤️‍🔥',
	// Objects & Symbols
	fire: '🔥', star: '⭐', sparkles: '✨', zap: '⚡', rainbow: '🌈',
	sun: '☀️', moon: '🌙', cloud: '☁️', snowflake: '❄️', umbrella: '☂️',
	100: '💯', checkmark: '✅', x: '❌', warning: '⚠️', question: '❓',
	exclamation: '❗', plus: '➕', minus: '➖', trophy: '🏆', medal: '🏅',
	crown: '👑', gem: '💎', bell: '🔔', music: '🎵', notes: '🎶',
	microphone: '🎤', headphones: '🎧', guitar: '🎸', trumpet: '🎺', drum: '🥁',
	art: '🎨', camera: '📷', film: '🎬', tv: '📺', computer: '💻',
	phone: '📱', keyboard: '⌨️', mouse: '🖱️', printer: '🖨️', bulb: '💡',
	book: '📖', books: '📚', bookmark: '🔖', link: '🔗', paperclip: '📎',
	lock: '🔒', unlock: '🔓', key: '🔑', hammer: '🔨', wrench: '🔧',
	gear: '⚙️', shield: '🛡️', bomb: '💣', knife: '🔪', gun: '🔫',
	pill: '💊', syringe: '💉', dna: '🧬', microscope: '🔬', telescope: '🔭',
	satellite: '📡', rocket: '🚀', airplane: '✈️', car: '🚗', bus: '🚌',
	// Food & Drink
	pizza: '🍕', hamburger: '🍔', fries: '🍟', hotdog: '🌭', taco: '🌮',
	burrito: '🌯', sushi: '🍣', ramen: '🍜', spaghetti: '🍝', cookie: '🍪',
	cake: '🎂', ice_cream: '🍦', donut: '🍩', chocolate: '🍫', candy: '🍬',
	popcorn: '🍿', coffee: '☕', tea: '🍵', beer: '🍺', wine: '🍷',
	cocktail: '🍸', champagne: '🍾', tropical_drink: '🍹',
	apple: '🍎', banana: '🍌', grapes: '🍇', watermelon: '🍉', strawberry: '🍓',
	peach: '🍑', avocado: '🥑', eggplant: '🍆', corn: '🌽', carrot: '🥕',
	// Animals
	dog: '🐕', cat: '🐈', mouse_face: '🐭', hamster: '🐹', rabbit: '🐰',
	fox: '🦊', bear: '🐻', panda: '🐼', koala: '🐨', lion: '🦁',
	cow: '🐄', pig: '🐷', frog: '🐸', monkey: '🐵', chicken: '🐔',
	penguin: '🐧', bird: '🐦', eagle: '🦅', owl: '🦉', bat: '🦇',
	wolf: '🐺', horse: '🐴', unicorn: '🦄', bee: '🐝', bug: '🐛',
	butterfly: '🦋', snail: '🐌', octopus: '🐙', crab: '🦀', shrimp: '🦐',
	whale: '🐋', dolphin: '🐬', fish: '🐟', shark: '🦈', turtle: '🐢',
	snake: '🐍', dragon: '🐉', dinosaur: '🦕',
	// People & Activities
	thumbs_up: '👍', eyes: '👀', brain: '🧠', tongue: '👅', ear: '👂',
	nose: '👃', foot: '🦶', bone: '🦴', baby: '👶', person: '🧑',
	man: '👨', woman: '👩', ghost: '👻', alien: '👽', robot: '🤖',
	poop: '💩', santa: '🎅', mermaid: '🧜', elf: '🧝', genie: '🧞',
	zombie: '🧟', vampire: '🧛', dancer: '💃', running: '🏃', walking: '🚶',
	surfing: '🏄', swimming: '🏊', basketball: '🏀', football: '🏈', soccer: '⚽',
	baseball: '⚾', tennis: '🎾', bowling: '🎳', golf: '⛳', boxing: '🥊',
	// Flags & Misc
	checkered_flag: '🏁', triangular_flag: '🚩', white_flag: '🏳️', rainbow_flag: '🏳️‍🌈',
	pirate_flag: '🏴‍☠️',
	// Party & Celebration
	tada: '🎉', confetti: '🎊', balloon: '🎈', party_popper: '🎉', party: '🥳',
	gift: '🎁', ribbon: '🎀', christmas_tree: '🎄', fireworks: '🎆', sparkler: '🎇',
	jack_o_lantern: '🎃', egg: '🥚',
	// Nature
	tree: '🌳', palm_tree: '🌴', cactus: '🌵', flower: '🌸', rose: '🌹',
	sunflower: '🌻', tulip: '🌷', seedling: '🌱', herb: '🌿', shamrock: '☘️',
	leaf: '🍃', maple_leaf: '🍁', fallen_leaf: '🍂', mushroom: '🍄',
	// Weather & Space
	earth: '🌍', globe: '🌐', volcano: '🌋', wave_emoji: '🌊', tornado: '🌪️',
	comet: '☄️', star2: '🌟', dizzy: '💫', boom: '💥', droplet: '💧',
	sweat_drops: '💦', dash: '💨',
};

// Build a searchable array for autocomplete
export const EMOJI_LIST = Object.entries(EMOJI_MAP).map(([name, emoji]) => ({
	name,
	emoji,
}));

export function searchEmoji(query: string, limit = 10): { name: string; emoji: string }[] {
	const q = query.toLowerCase();
	// Exact prefix matches first, then contains
	const prefixMatches: { name: string; emoji: string }[] = [];
	const containsMatches: { name: string; emoji: string }[] = [];
	for (const entry of EMOJI_LIST) {
		if (entry.name.startsWith(q)) {
			prefixMatches.push(entry);
		} else if (entry.name.includes(q)) {
			containsMatches.push(entry);
		}
		if (prefixMatches.length + containsMatches.length >= limit) break;
	}
	return [...prefixMatches, ...containsMatches].slice(0, limit);
}
