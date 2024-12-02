# Run K6 testing.
API_URL=http://localhost:35124/api k6 run --summary-trend-stats="avg,min,max,med,p(95),p(99),p(99.9)" script.js