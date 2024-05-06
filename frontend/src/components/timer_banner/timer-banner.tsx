import { useEffect, useState } from "react";



function TimerBanner({ timeLeft }: { timeLeft: number }) {
    const [progress, setProgress] = useState(timeLeft);

    useEffect(() => {
        let timer = setInterval(() => {
            setProgress((prevProgress) => {
                if (prevProgress === 0) {
                    clearInterval(timer);
                    return 0;
                } else {
                    return prevProgress - 10;
                }
            })
        }, 1000)

        return () => clearInterval(timer);
    });

    return (
        <div>
            <progress value={progress} max="100" style={{ width: '100%', height: '30px' }} />
        </div>
    );
}

export default TimerBanner;

