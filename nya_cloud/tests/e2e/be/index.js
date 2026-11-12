const express = require('express');
const cors = require('cors');

const app = express();
app.use(cors());

app.get('/hello', (req, res) => {
    res.json("Hello World E2E!");
});

app.get('/:data', (req, res) => {
    res.json(req.params.data);
});


const PORT = process.env.PORT || 80;
app.listen(PORT, () => {
    console.log(`Backend server is running on port ${PORT}`);
});