const LineReader = require('readline');
const fs = require('fs');

const TIME_REGEX = /(\w+) (\d+\.\d+)µs/;

const bucketValues = {};
const bucketCounts = {};

const linereader = LineReader.createInterface({
  input: fs.createReadStream('./bench.log')
})

linereader.on('line', (line) => {
  const match = line.match(TIME_REGEX);

  if (match) {
    const key = match[1];
    const value = match[2];

    const bucketValue = bucketValues[key] || 0;
    const bucketCount = bucketCounts[key] || 0;
    bucketValues[key] = bucketValue + Number(value);
    bucketCounts[key] = bucketCount + 1;
  }
});

linereader.on('close', () => {
  for (let key in bucketValues) {
    console.log(`${key}: ${bucketValues[key] / bucketCounts[key]}µs`);
  }
});
