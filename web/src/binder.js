// @ts-check
/// <reference lib="dom" />

/** @type { (ctx: number, chunk: number, fun_id: number, event: Event) => void } */
let event_listener = /** @type {any} */ (null);
/** @type { (fun: (ctx: number, chunk: number, fun_id: number, event: Event) => void) => void } */
export function register_event_callback (fun) {
	event_listener = fun;
}

/** @typedef { { ind: number } } Cursor */

/** @type { (buf: Uint8Array, cur: Cursor) => number } */
function decode_u8 (buf, cur) {
	return buf[cur.ind++]
}

/** decode LEB128 unsigned integer
	@type { (buf: Uint8Array, cur: Cursor) => number } */
function decode_vuint (buf, cur) {
	let res = 0, shift = 0, cond = true;
	while (cond) {
		let byte = decode_u8(buf, cur);
		res |= (byte & 0b0111_1111) << shift;
		shift += 7;
		cond = (byte & 0b1000_0000) != 0;
	}
	return res;
}

let text_decoder = new TextDecoder;
/** @type { (buf: Uint8Array, cur: Cursor) => string } */
function decode_str (buf, cur) {
	let len = decode_vuint(buf, cur);
	let text = text_decoder.decode(buf.subarray(cur.ind, cur.ind + len));
	cur.ind += len;
	return text;
}

/** @type { (buf: Uint8Array, cur: Cursor) => string } */
function decode_name (buf, cur) {
	let tag = decode_vuint(buf, cur);
	if ((tag & 1) === 1) {
		return common_names[tag >> 1]
	}
	let len = tag >> 1;
	let text = text_decoder.decode(buf.subarray(cur.ind, cur.ind + len));
	cur.ind += len;
	return text;
}

const EL_START = 0;
const EL_ID = 1;
const ATTR = 2;
const PROP = 3;
const CLASS = 4;
const STYLE = 5;
const TEXT = 6;
const NODE = 7;
const EVENT = 8;
const END = 255;

/** @type { (
	el: HTMLElement, build_codes: Uint8Array, cur: Cursor, el_refs: Element[], props: any[], nodes: Node[]
) => void } */
function construct_el (el, build_codes, cur, el_refs, props, nodes) {
	while (true) {
		let op = decode_u8(build_codes, cur);
		switch (op) {
			case EL_START: {
				let tag = decode_name(build_codes, cur);
				let child = document.createElement(tag)
				construct_el(child, build_codes, cur, el_refs, props, nodes);
				el.append(child);
				break
			}
			case EL_ID: {
				el_refs.push(el);
				break
			}
			case ATTR: {
				let attr = decode_name(build_codes, cur);
				let value = decode_str(build_codes, cur);
				el.setAttribute(attr, value);
				break
			}
			case PROP: {
				let prop = decode_str(build_codes, cur);
				let ind = decode_vuint(build_codes, cur);
				/** @type {any} */ (el)[prop] = props[ind];
				break
			}
			case CLASS: {
				let name = decode_str(build_codes, cur);
				el.classList.add(name);
				break
			}
			case STYLE: {
				let prop = decode_name(build_codes, cur);
				let value = decode_str(build_codes, cur);
				el.style.setProperty(prop, value)
				break
			}
			case TEXT: {
				let text = decode_str(build_codes, cur);
				el.append(text);
				break
			}
			case NODE: {
				let ind = decode_vuint(build_codes, cur);
				el.append(nodes[ind])
				break
			}
			case EVENT: {
				let ctx = decode_vuint(build_codes, cur);
				let chunk = decode_vuint(build_codes, cur);
				let name = decode_name(build_codes, cur);
				let fun_id = decode_vuint(build_codes, cur);
				el.addEventListener(name, (event) => event_listener(ctx, chunk, fun_id, event));
				break
			}
			case END: return;
			default: throw `binder: unkown opcode ${op}`
		}
	}
}

/** contruct an element from build codes
  @type { (
	target_el: HTMLElement, build_codes: Uint8Array, props: any[], nodes: Node[]
  ) => Element[] } */
export function construct (target_el, build_codes, props, nodes) {
	let cur = { ind: 0 };
	let el_refs = [target_el];
	construct_el(target_el, build_codes, cur, el_refs, props, nodes);
	if (cur.ind !== build_codes.length) throw "binder: excess input";
	return el_refs;
}
