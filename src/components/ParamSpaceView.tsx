import { type Curve, CURVE_COLORS, type Point } from '../curves';

interface ParamSpaceViewProps {
    curves: [Curve, Curve];
}

/**
 * Computes Euclidean distance two points.
 */
function dist(p1: Point, p2: Point): number {
    return Math.sqrt((p1.x - p2.x) ** 2 + (p1.y - p2.y) ** 2);
}

/**
 * Computes the cumulative arc length of the curve up to each point.
 */
function computeCumulativeLengths(curve: Curve): number[] {
    let cumulativeLength = 0;
    return curve.map((point, pointIdx) => {
        if (pointIdx !== 0) {
            cumulativeLength += dist(point, curve[pointIdx - 1]);
        }
        return cumulativeLength;
    });
}

export function ParamSpaceView(props: ParamSpaceViewProps): JSX.Element {
    const { curves } = props;

    if (curves.some((curve) => curve.length <= 1)) {
        return null;
    }

    const [cumulativeLengths1, cumulativeLengths2] = curves.map(
        computeCumulativeLengths,
    );

    const width = cumulativeLengths1[cumulativeLengths1.length - 1];
    const height = cumulativeLengths2[cumulativeLengths2.length - 1];

    return (
        <svg
            width={width + 30}
            height={height + 30}
            style={{ border: '1px solid gray' }}
        >
            {cumulativeLengths2.map((y, pointIdx) => (
                <circle
                    key={pointIdx}
                    cx={10}
                    cy={y + 20}
                    r={5}
                    fill={CURVE_COLORS[1]}
                />
            ))}
            {cumulativeLengths1.map((x, pointIdx) => (
                <circle
                    key={pointIdx}
                    cx={x + 20}
                    cy={10}
                    r={5}
                    fill={CURVE_COLORS[0]}
                />
            ))}
        </svg>
    );
}
