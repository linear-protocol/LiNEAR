"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.fetchJson = void 0;
const types_1 = require("@near-js/types");
const http_errors_1 = __importDefault(require("http-errors"));
const exponential_backoff_1 = require("./exponential-backoff");
const START_WAIT_TIME_MS = 1000;
const BACKOFF_MULTIPLIER = 1.5;
const RETRY_NUMBER = 10;
const logWarning = (...args) => !process.env['NEAR_NO_LOGS'] && console.warn(...args);
function fetchJson(connectionInfoOrUrl, json) {
    return __awaiter(this, void 0, void 0, function* () {
        let connectionInfo = { url: null };
        if (typeof (connectionInfoOrUrl) === 'string') {
            connectionInfo.url = connectionInfoOrUrl;
        }
        else {
            connectionInfo = connectionInfoOrUrl;
        }
        const response = yield (0, exponential_backoff_1.exponentialBackoff)(START_WAIT_TIME_MS, RETRY_NUMBER, BACKOFF_MULTIPLIER, () => __awaiter(this, void 0, void 0, function* () {
            try {
                if (!global.fetch) {
                    global.fetch = (yield Promise.resolve().then(() => __importStar(require('./fetch')))).default;
                }
                const response = yield global.fetch(connectionInfo.url, {
                    method: json ? 'POST' : 'GET',
                    body: json ? json : undefined,
                    headers: Object.assign(Object.assign({}, connectionInfo.headers), { 'Content-Type': 'application/json' })
                });
                if (!response.ok) {
                    if (response.status === 503) {
                        logWarning(`Retrying HTTP request for ${connectionInfo.url} as it's not available now`);
                        return null;
                    }
                    // throw (0, http_errors_1.default)(response.status, yield response.text());
                    console.error("fetch error", response.status, yield response.text());
                    console.error("retrying...");
                    return null;
                }
                return response;
            }
            catch (error) {
                if (error.toString().includes('FetchError') || error.toString().includes('Failed to fetch')) {
                    logWarning(`Retrying HTTP request for ${connectionInfo.url} because of error: ${error}`);
                    return null;
                }
                throw error;
            }
        }));
        if (!response) {
            throw new types_1.TypedError(`Exceeded ${RETRY_NUMBER} attempts for ${connectionInfo.url}.`, 'RetriesExceeded');
        }
        return yield response.json();
    });
}
exports.fetchJson = fetchJson;
