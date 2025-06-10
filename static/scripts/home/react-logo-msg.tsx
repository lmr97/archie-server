import { useState } from 'react';
import { motion } from "motion/react"

export default function ReactLogoMessage() {


    function viewBoxParams(size: number) {
        const offset = (-size) / 2;
        return `${offset} ${offset} ${size} ${size}`
    }

    function DynMessage() {
        return (
            <motion.p className="react-msg" 
            variants={{
                mouseAway: { 
                    x: [null, 80], 
                    opacity: 0
                },
                hoveredOver: { 
                    x: [null, -10], 
                    opacity: 1,
                },
            }}
            transition={{ duration: 1 }}
            >
                Made with love (and React)
            </motion.p>
        )
    }

    function ReactLogo() {
        return (
            <motion.svg
                data-testid="active-react-logo"
                viewBox={viewBoxParams(30)}
                xmlns="http://www.w3.org/2000/svg" 
                className="react-logo"
                variants={{
                    hoveredOver: {
                        viewBox: viewBoxParams(27),
                        transition: { duration: 1 }
                    },
                    mouseAway:   {
                        viewBox: viewBoxParams(30),
                        transition: { duration: 1 }
                    },
                }}
                // transition={{ duration: 1 }}
                >
                <circle cx="0" cy="0" r="2" fill="currentColor"></circle>
                <g stroke="currentColor" strokeWidth="1" fill="none">
                    <ellipse rx="10" ry="4.5"></ellipse>
                    <ellipse rx="10" ry="4.5" transform="rotate(60)"></ellipse>
                    <ellipse rx="10" ry="4.5" transform="rotate(120)"></ellipse>
                </g>
            </motion.svg>
        )
    }

    // React's logo is rendered in real time as an SVG
    // on their own page, so the shape is copied directly
    // from react.dev
    return (
        <motion.div 
            className="interact-synchronizer"
            data-testid="react-logo-div"
            // placing state setting here because hovering
            // is only detected on the actual SVG's lines 
            // and figures if I place it in the inner <svg>
            initial="mouseAway"
            animate="mouseAway"
            whileHover="hoveredOver"
            >
            <DynMessage />
            <motion.svg 
                id="react-logo-msg" 
                viewBox="0 0 30 30"
                >
                <ReactLogo />
            </motion.svg>
        </motion.div>
    )
}