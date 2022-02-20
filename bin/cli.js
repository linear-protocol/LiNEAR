#!/usr/bin/env node
require('yargs/yargs')(process.argv.slice(2))
  .commandDir('commands')
  .demandCommand()
  .help()
  .argv
