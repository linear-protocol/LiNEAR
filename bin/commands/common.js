const prompts = require('prompts');
const base58 = require('bs58');
const sha256 = require('sha256');

exports.networkOption = {
  describe: 'network ID',
  default: 'testnet',
  choices: ['testnet', 'mainnet', 'localnet']
};

exports.doubleCheck = async () => {
  const res = await prompts({
    type: 'toggle',
    name: 'value',
    message: 'Confirm?',
    initial: true,
    active: 'yes',
    inactive: 'no'
  });
  if (!res.value) process.exit(1);
}

exports.parseHashReturnValue = (outcome) => {
  const status = outcome.status;
  const data = status.SuccessValue;
  if (!data) {
    throw new Error('bad return value');
  }

  const buff = Buffer.from(data, 'base64');
  return buff.toString('ascii').replaceAll('"', "");
}

exports.getBase58CodeHash = (code) => {
  const hash = Buffer.from(sha256(code), 'hex');
  return base58.encode(hash);
}
