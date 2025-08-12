import http from 'node:http'

http.get('http://localhost:8000', res => {
  let data = ''
  res.on('data', chunk => { data += chunk })
  res.on('end', () => {
    console.log(data)
  })
}).on('error', err => {
  console.error('Request failed:', err)
})
